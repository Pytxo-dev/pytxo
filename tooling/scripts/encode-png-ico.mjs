const PNG_SIGNATURE = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

function pngDimensions(buffer) {
  if (!Buffer.isBuffer(buffer) || buffer.length < 24 || !buffer.subarray(0, 8).equals(PNG_SIGNATURE)) {
    throw new Error("ICO input must be a PNG buffer");
  }
  const width = buffer.readUInt32BE(16);
  const height = buffer.readUInt32BE(20);
  if (!width || width > 256 || width !== height) {
    throw new Error(`ICO PNG must be square and 1-256px; received ${width}x${height}`);
  }
  return width;
}

/** Encode PNG images directly into a modern ICO container without reprocessing pixels. */
export function encodePngIco(images) {
  if (!Array.isArray(images) || images.length === 0 || images.length > 0xffff) {
    throw new Error("ICO requires between 1 and 65535 PNG images");
  }

  const normalized = images.map((image) => ({ image, size: pngDimensions(image) }));
  const headerSize = 6 + normalized.length * 16;
  const header = Buffer.alloc(headerSize);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(normalized.length, 4);

  let offset = headerSize;
  normalized.forEach(({ image, size }, index) => {
    const entry = 6 + index * 16;
    header.writeUInt8(size === 256 ? 0 : size, entry);
    header.writeUInt8(size === 256 ? 0 : size, entry + 1);
    header.writeUInt8(0, entry + 2);
    header.writeUInt8(0, entry + 3);
    header.writeUInt16LE(1, entry + 4);
    header.writeUInt16LE(32, entry + 6);
    header.writeUInt32LE(image.length, entry + 8);
    header.writeUInt32LE(offset, entry + 12);
    offset += image.length;
  });

  return Buffer.concat([header, ...normalized.map(({ image }) => image)]);
}
