import assert from "node:assert/strict";
import { mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { isRedistributableRuntime, parsePeImports, verifyWindowsRuntime } from "./verify-windows-runtime.mjs";

// Small PE images with real section/RVA layouts. No compiler or executable code
// is needed to exercise import-directory decoding; these are never executed.
function pe({ bits = 64, normal = [], delay = [], delayAttributes = 1, legacyDelay = false } = {}) {
  const bytes = Buffer.alloc(0xa00);
  bytes.write("MZ");
  bytes.writeUInt32LE(0x80, 0x3c);
  bytes.write("PE\0\0", 0x80);
  bytes.writeUInt16LE(bits === 64 ? 0x8664 : 0x14c, 0x84);
  bytes.writeUInt16LE(1, 0x86);
  const optional = 0x98;
  const optionalSize = bits === 64 ? 240 : 224;
  bytes.writeUInt16LE(optionalSize, 0x94);
  bytes.writeUInt16LE(2, 0x96);
  bytes.writeUInt16LE(bits === 64 ? 0x20b : 0x10b, optional);
  if (bits === 64) bytes.writeBigUInt64LE(0x140000000n, optional + 24);
  else bytes.writeUInt32LE(0x400000, optional + 28);
  bytes.writeUInt32LE(0x2000, optional + 56);
  bytes.writeUInt32LE(0x200, optional + 60);
  const directories = optional + (bits === 64 ? 112 : 96);
  bytes.writeUInt32LE(16, directories - 4);
  const section = optional + optionalSize;
  bytes.write(".rdata", section);
  bytes.writeUInt32LE(0x800, section + 8);
  bytes.writeUInt32LE(0x1000, section + 12);
  bytes.writeUInt32LE(0x800, section + 16);
  bytes.writeUInt32LE(0x200, section + 20);
  let stringOffset = 0x600;
  const rva = offset => 0x1000 + offset - 0x200;
  for (const [names, index, start, stride, nameOffset] of [[normal, 1, 0x200, 20, 12], [delay, 13, 0x400, 32, 4]]) {
    if (!names.length) continue;
    bytes.writeUInt32LE(rva(start), directories + index * 8);
    bytes.writeUInt32LE((names.length + 1) * stride, directories + index * 8 + 4);
    names.forEach((name, i) => {
      if (index === 13) bytes.writeUInt32LE(delayAttributes, start + i * stride);
      bytes.writeUInt32LE(rva(stringOffset) + (index === 13 && legacyDelay ? 0x400000 : 0), start + i * stride + nameOffset);
      bytes.write(name, stringOffset, "ascii");
      stringOffset += Buffer.byteLength(name) + 1;
    });
  }
  return { bytes, optional, directories, section };
}

for (const bits of [32, 64]) {
  test(`decodes PE${bits} ordinary and delay imports using section RVAs`, () => {
    const { bytes } = pe({ bits, normal: ["KERNEL32.dll", "MSVCP140.dll"], delay: ["VCRUNTIME140_1.dll"] });
    assert.deepEqual(parsePeImports(bytes).imports, [
      { name: "KERNEL32.dll", kind: "normal" },
      { name: "MSVCP140.dll", kind: "normal" },
      { name: "VCRUNTIME140_1.dll", kind: "delay" },
    ]);
  });
}

test("supports the earlier PE32 delay descriptor's image-base-relative conversion", () => {
  const { bytes } = pe({ bits: 32, delay: ["msvcr120.dll"], delayAttributes: 0, legacyDelay: true });
  assert.deepEqual(parsePeImports(bytes).imports, [{ name: "msvcr120.dll", kind: "delay" }]);
});

test("rejects unsupported delay attributes and PE32+ legacy addresses", () => {
  assert.throws(() => parsePeImports(pe({ delay: ["x.dll"], delayAttributes: 2 }).bytes), /delay.*attributes/i);
  assert.throws(() => parsePeImports(pe({ delay: ["x.dll"], delayAttributes: 0 }).bytes), /legacy.*PE32\+/i);
});

test("rejects legacy delay addresses below the image base", () => {
  assert.throws(() => parsePeImports(pe({ bits: 32, delay: ["x.dll"], delayAttributes: 0 }).bytes), /image base/i);
});

test("rejects case-insensitive versioned runtime families and suffixed/debug variants", () => {
  for (const name of ["MSVCP140.dll", "msvcp140_atomic_wait.dll", "msvcp140_codecvt_ids.dll", "VCRUNTIME140_1.dll", "msvcr120d.dll", "msvcm90.dll", "concrt140.dll", "vccorlib140.dll", "vcomp140.dll", "vcamp140.dll", "ucrtbased.dll", "msvcrtd.dll", "C:\\runtime\\MSVCP140.dll", "runtime/msvcp140.dll"]) {
    assert.equal(isRedistributableRuntime(name), true, name);
  }
});

for (const kind of ["normal", "delay"]) {
  test(`rejects extensionless MSVC runtimes in ${kind} imports`, async () => {
    const root = await mkdtemp(path.join(tmpdir(), "pytxo-pe-extensionless-"));
    const file = path.join(root, "bad.exe");
    await writeFile(file, pe({ [kind]: ["MSVCP140", "vcruntime140_1"] }).bytes);
    const result = await verifyWindowsRuntime([file]);
    assert.equal(result.ok, false);
    assert.deepEqual(result.files[0].forbiddenImports, [
      { name: "MSVCP140", kind },
      { name: "vcruntime140_1", kind },
    ]);
  });
}

test("distinguishes Windows system CRT names from versioned redistributables", () => {
  for (const name of ["kernel32.dll", "msvcrt.dll", "msvcp_win.dll", "ucrtbase.dll", "api-ms-win-crt-runtime-l1-1-0.dll", "MSVCRT", "msvcp_win", "ucrtbase", "api-ms-win-crt-runtime-l1-1-0"]) {
    assert.equal(isRedistributableRuntime(name), false, name);
  }
});

test("does not interpret an unrelated runtime-name string as an import", () => {
  const { bytes } = pe({ normal: ["KERNEL32.dll"] });
  bytes.write("MSVCP140.dll\0", 0x800);
  assert.deepEqual(parsePeImports(bytes).imports, [{ name: "KERNEL32.dll", kind: "normal" }]);
});

test("accepts images with absent import directories", () => {
  assert.deepEqual(parsePeImports(pe().bytes).imports, []);
});

test("rejects non-PE, truncated, and unsupported optional headers", () => {
  assert.throws(() => parsePeImports(Buffer.from("MSVCP140.dll")), /PE|DOS|truncated/i);
  assert.throws(() => parsePeImports(pe().bytes.subarray(0, 0x100)), /truncated|header/i);
  const { bytes, optional } = pe();
  bytes.writeUInt16LE(0x107, optional);
  assert.throws(() => parsePeImports(bytes), /optional.*magic/i);
});

test("rejects directory RVAs outside the file-backed section", () => {
  const { bytes, directories, section } = pe({ normal: ["x.dll"] });
  bytes.writeUInt32LE(0x3000, directories + 8);
  assert.throws(() => parsePeImports(bytes), /RVA|file-backed/i);
  bytes.writeUInt32LE(0x1900, directories + 8);
  bytes.writeUInt32LE(0x1000, section + 8);
  assert.throws(() => parsePeImports(bytes), /RVA|file-backed/i);
});

test("rejects truncated and unterminated import descriptor tables", () => {
  for (const [kind, index, stride] of [["normal", 1, 20], ["delay", 13, 32]]) {
    const { bytes, directories } = pe({ [kind]: ["x.dll"] });
    bytes.writeUInt32LE(stride - 1, directories + index * 8 + 4);
    assert.throws(() => parsePeImports(bytes), /descriptor|terminat/i);
    bytes.writeUInt32LE(stride, directories + index * 8 + 4);
    assert.throws(() => parsePeImports(bytes), /terminat/i);
  }
});

test("does not treat a partially zero descriptor as the end of the table", () => {
  const { bytes } = pe({ normal: ["x.dll"] });
  bytes.writeUInt32LE(0, 0x20c);
  bytes.writeUInt32LE(1, 0x200);
  assert.throws(() => parsePeImports(bytes), /name/i);
});

test("rejects inconsistent absent directories and oversized directory counts", () => {
  const { bytes, directories } = pe();
  bytes.writeUInt32LE(20, directories + 12);
  assert.throws(() => parsePeImports(bytes), /directory/i);
  bytes.writeUInt32LE(0, directories + 12);
  bytes.writeUInt32LE(17, directories - 4);
  assert.throws(() => parsePeImports(bytes), /director|optional/i);
});

test("rejects a DLL name outside the section, missing its terminator, or containing control bytes", () => {
  const { bytes } = pe({ normal: ["x.dll"] });
  bytes.writeUInt32LE(0x1800, 0x20c);
  assert.throws(() => parsePeImports(bytes), /RVA|file-backed/i);
  bytes.writeUInt32LE(0x17fe, 0x20c);
  bytes.write("xx", 0x9fe);
  assert.throws(() => parsePeImports(bytes), /terminat/i);
  bytes[0x9fe] = 1;
  bytes[0x9ff] = 0;
  assert.throws(() => parsePeImports(bytes), /name/i);
});

test("rejects overlapping virtual sections instead of choosing a convenient mapping", () => {
  const { bytes, section } = pe({ normal: ["x.dll"] });
  bytes.writeUInt16LE(2, 0x86);
  bytes.copy(bytes, section + 40, section, section + 40);
  assert.throws(() => parsePeImports(bytes), /overlap/i);
});

test("reports hashes and failures for every explicit file, including delay-only dependencies", async () => {
  const root = await mkdtemp(path.join(tmpdir(), "pytxo-pe-"));
  const clean = pe({ normal: ["KERNEL32.dll"] }).bytes;
  const bad = pe({ delay: ["MSVCP140.dll"] }).bytes;
  const cleanFile = path.join(root, "clean.exe");
  const badFile = path.join(root, "bad.exe");
  const malformedFile = path.join(root, "malformed.exe");
  await writeFile(cleanFile, clean);
  await writeFile(badFile, bad);
  await writeFile(malformedFile, "not a PE");
  // A neighboring runtime DLL must not create an exemption from static policy.
  await writeFile(path.join(root, "MSVCP140.dll"), clean);
  const result = await verifyWindowsRuntime([cleanFile, badFile, malformedFile, path.join(root, "missing.exe")]);
  assert.equal(result.ok, false);
  assert.deepEqual(result.files.map(file => file.ok), [true, false, false, false]);
  assert.equal(result.files[0].sha256, createHash("sha256").update(clean).digest("hex"));
  assert.deepEqual(result.files[1].forbiddenImports, [{ name: "MSVCP140.dll", kind: "delay" }]);
  assert.match(result.files[2].error, /PE|DOS|truncated/i);
  assert.equal(result.files[2].sha256, createHash("sha256").update("not a PE").digest("hex"));
  assert.match(result.files[3].error, /ENOENT/);
});

test("CLI exits nonzero with JSON evidence and requires explicit files", async () => {
  const root = await mkdtemp(path.join(tmpdir(), "pytxo-pe-cli-"));
  const file = path.join(root, "bad.exe");
  await writeFile(file, pe({ normal: ["MSVCP140.dll"] }).bytes);
  const script = fileURLToPath(new URL("./verify-windows-runtime.mjs", import.meta.url));
  const child = spawnSync(process.execPath, [script, file], { encoding: "utf8" });
  assert.equal(child.status, 1);
  assert.equal(JSON.parse(child.stdout).files[0].forbiddenImports[0].name, "MSVCP140.dll");
  const usage = spawnSync(process.execPath, [script], { encoding: "utf8" });
  assert.equal(usage.status, 1);
  assert.match(usage.stderr, /Usage/);
});
