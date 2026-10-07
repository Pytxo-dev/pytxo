import assert from "node:assert/strict";
import test from "node:test";

import { encodePngIco } from "./encode-png-ico.mjs";

function pngHeader(size) {
  const value = Buffer.alloc(24);
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]).copy(value);
  value.writeUInt32BE(size, 16);
  value.writeUInt32BE(size, 20);
  return value;
}

test("encodes ordered PNG images with valid ICO directory offsets", () => {
  const png16 = pngHeader(16);
  const png256 = pngHeader(256);
  const ico = encodePngIco([png16, png256]);
  assert.equal(ico.readUInt16LE(0), 0);
  assert.equal(ico.readUInt16LE(2), 1);
  assert.equal(ico.readUInt16LE(4), 2);
  assert.equal(ico.readUInt8(6), 16);
  assert.equal(ico.readUInt8(22), 0);
  assert.equal(ico.readUInt32LE(18), 38);
  assert.equal(ico.readUInt32LE(34), 38 + png16.length);
  assert.deepEqual(ico.subarray(38, 38 + png16.length), png16);
  assert.deepEqual(ico.subarray(38 + png16.length), png256);
});

test("rejects malformed, non-square, oversized, and empty inputs", () => {
  assert.throws(() => encodePngIco([]), /between 1 and 65535/);
  assert.throws(() => encodePngIco([Buffer.from("not a png")]), /must be a PNG/);
  const nonSquare = pngHeader(16);
  nonSquare.writeUInt32BE(32, 20);
  assert.throws(() => encodePngIco([nonSquare]), /square and 1-256px/);
  assert.throws(() => encodePngIco([pngHeader(257)]), /square and 1-256px/);
});
