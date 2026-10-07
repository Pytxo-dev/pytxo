import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

// This release policy requires static MSVC runtimes. Finding a DLL on PATH or
// beside the image does not exempt an import. Windows' own unversioned CRT and
// UCRT API sets are distinct from these redistributable/debug dependencies.
export function isRedistributableRuntime(name) {
  const basename = path.win32.basename(name.replaceAll("/", "\\"));
  // Windows library loading supplies .dll when the requested name has no
  // extension; retain the original import spelling in the evidence report.
  const library = path.win32.extname(basename) ? basename : `${basename}.dll`;
  return /^(?:(?:msvc[pmr]|vcruntime|vccorlib|concrt|vcomp|vcamp)\d[a-z0-9_]*|msvcrtd|ucrtbased)\.dll$/i.test(library);
}

// Decode only import-directory metadata, without loading or executing the PE.
// https://learn.microsoft.com/en-us/windows/win32/debug/pe-format
// Delay attributes follow delayimp.h (dlattrRva=1), as documented at:
// https://learn.microsoft.com/en-us/cpp/build/reference/understanding-the-helper-function
export function parsePeImports(bytes) {
  if (!Buffer.isBuffer(bytes)) throw new TypeError("PE input must be a Buffer");
  function range(offset, size, label) {
    if (!Number.isSafeInteger(offset) || !Number.isSafeInteger(size) || offset < 0 || size < 0 || offset + size > bytes.length) {
      throw new Error(`Truncated or invalid ${label}`);
    }
  }
  range(0, 64, "DOS header");
  if (bytes.toString("ascii", 0, 2) !== "MZ") throw new Error("Missing DOS MZ signature");
  const pe = bytes.readUInt32LE(0x3c);
  if (pe < 64) throw new Error("Invalid PE header offset");
  range(pe, 24, "PE header");
  if (bytes.readUInt32LE(pe) !== 0x00004550) throw new Error("Missing PE signature");
  const machine = bytes.readUInt16LE(pe + 4);
  const sectionCount = bytes.readUInt16LE(pe + 6);
  if (sectionCount > 96) throw new Error("Unsupported PE section count");
  const optionalSize = bytes.readUInt16LE(pe + 20);
  const optional = pe + 24;
  range(optional, optionalSize, "optional header");
  if (optionalSize < 2) throw new Error("Truncated optional header");
  const magic = bytes.readUInt16LE(optional);
  if (magic !== 0x10b && magic !== 0x20b) throw new Error("Unsupported optional header magic");
  const is64 = magic === 0x20b;
  const directoryOffset = is64 ? 112 : 96;
  if (optionalSize < directoryOffset) throw new Error("Truncated optional header directories");
  const imageBase = is64 ? bytes.readBigUInt64LE(optional + 24) : BigInt(bytes.readUInt32LE(optional + 28));
  const directoryCount = bytes.readUInt32LE(optional + directoryOffset - 4);
  if (directoryCount > 16 || directoryOffset + directoryCount * 8 > optionalSize) {
    throw new Error("Unsupported or truncated optional header directories");
  }
  const headersSize = bytes.readUInt32LE(optional + 60);
  const sectionTable = optional + optionalSize;
  range(sectionTable, sectionCount * 40, "section headers");
  if (headersSize < sectionTable + sectionCount * 40 || headersSize > bytes.length) {
    throw new Error("Invalid PE SizeOfHeaders");
  }
  const sections = [];
  for (let i = 0; i < sectionCount; i++) {
    const start = sectionTable + i * 40;
    const virtualSize = bytes.readUInt32LE(start + 8);
    const rva = bytes.readUInt32LE(start + 12);
    const rawSize = bytes.readUInt32LE(start + 16);
    const raw = bytes.readUInt32LE(start + 20);
    const virtualEnd = rva + Math.max(virtualSize, rawSize);
    if (virtualEnd > 0x100000000) throw new Error("PE section RVA overflow");
    if (rawSize) {
      range(raw, rawSize, "section raw data");
      if (raw < headersSize) throw new Error("Section raw data overlaps PE headers");
    }
    if (virtualEnd > rva) {
      if (rva < headersSize || sections.some(section => rva < section.virtualEnd && virtualEnd > section.rva)) {
        throw new Error("Overlapping PE virtual sections or headers");
      }
      sections.push({ rva, virtualEnd, raw, rawSize });
    }
  }

  function mapped(rva, size, label) {
    if (!Number.isSafeInteger(rva) || rva < 0 || rva + size > 0x100000000) throw new Error(`Invalid ${label} RVA`);
    if (rva < headersSize && rva + size <= headersSize) return { offset: rva, end: headersSize };
    const section = sections.find(section => rva >= section.rva && rva < section.virtualEnd);
    if (!section || rva - section.rva + size > section.rawSize) throw new Error(`${label} RVA is not wholly file-backed`);
    return { offset: section.raw + rva - section.rva, end: section.raw + section.rawSize };
  }

  function dllName(rva) {
    if (!rva) throw new Error("Import descriptor has no DLL name");
    const { offset, end } = mapped(rva, 1, "DLL name");
    const limit = Math.min(end, offset + 4096);
    const terminator = bytes.subarray(offset, limit).indexOf(0);
    if (terminator < 0) throw new Error("Unterminated or oversized DLL name");
    const name = bytes.toString("latin1", offset, offset + terminator);
    if (!/^[\x20-\x7e]+$/.test(name)) throw new Error("Unsupported or empty DLL name");
    return name;
  }

  const imports = [];
  for (const [kind, index, stride, nameOffset] of [["normal", 1, 20, 12], ["delay", 13, 32, 4]]) {
    if (index >= directoryCount) continue;
    const directory = optional + directoryOffset + index * 8;
    const rva = bytes.readUInt32LE(directory);
    const size = bytes.readUInt32LE(directory + 4);
    if (!rva && !size) continue;
    if (!rva || !size) throw new Error(`Inconsistent ${kind} import directory RVA/size`);
    if (size < stride) throw new Error(`Truncated ${kind} import descriptor table`);
    const { offset } = mapped(rva, size, `${kind} import directory`);
    const end = offset + size;
    let terminated = false;
    for (let descriptor = offset; descriptor + stride <= end; descriptor += stride) {
      if (bytes.subarray(descriptor, descriptor + stride).every(byte => byte === 0)) {
        terminated = true;
        break;
      }
      let nameRva = bytes.readUInt32LE(descriptor + nameOffset);
      if (kind === "delay") {
        const attributes = bytes.readUInt32LE(descriptor);
        if (attributes !== 0 && attributes !== 1) throw new Error("Unsupported delay import attributes");
        if (attributes === 0) {
          if (is64) throw new Error("Unsupported legacy delay addresses in PE32+");
          const relative = BigInt(nameRva) - imageBase;
          if (relative < 0n) throw new Error("Legacy delay DLL address is below the image base");
          nameRva = Number(relative);
        }
      }
      imports.push({ name: dllName(nameRva), kind });
    }
    if (!terminated) throw new Error(`Unterminated ${kind} import descriptor table`);
  }
  return { format: is64 ? "PE32+" : "PE32", machine, imports };
}

export async function verifyWindowsRuntime(files) {
  if (!Array.isArray(files) || !files.length || files.some(file => typeof file !== "string" || !file)) {
    throw new Error("Usage: verify-windows-runtime.mjs <executable-or-dll> [more files...]");
  }
  const results = [];
  // Inspect explicit inputs sequentially to keep memory bounded for large PEs.
  for (const file of files) {
    const record = { file: path.resolve(file), sha256: null, imports: null, ok: false };
    try {
      const bytes = await readFile(record.file);
      record.sha256 = createHash("sha256").update(bytes).digest("hex");
      Object.assign(record, parsePeImports(bytes));
      record.forbiddenImports = record.imports.filter(({ name }) => isRedistributableRuntime(name));
      record.ok = record.forbiddenImports.length === 0;
    } catch (error) {
      record.error = error.message;
    }
    results.push(record);
  }
  return {
    ok: results.every(file => file.ok),
    policy: "No versioned MSVC redistributable or debug CRT imports",
    scope: "Explicit PE import metadata only; runtime LoadLibrary dependencies and clean-machine behavior require separate verification",
    files: results,
  };
}

const invoked = process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;
if (invoked) {
  try {
    const result = await verifyWindowsRuntime(process.argv.slice(2));
    console.log(JSON.stringify(result, null, 2));
    if (!result.ok) process.exitCode = 1;
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
