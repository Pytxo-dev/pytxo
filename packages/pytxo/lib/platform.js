"use strict";

function getAssetName(platform = process.platform, arch = process.arch) {
  const assets = {
    "win32:x64": "pytxo-windows-x64.exe",
    "darwin:arm64": "pytxo-darwin-arm64",
    "darwin:x64": "pytxo-darwin-x64",
    "linux:arm64": "pytxo-linux-arm64",
    "linux:x64": "pytxo-linux-x64",
  };
  const asset = assets[`${platform}:${arch}`];
  if (!asset) {
    throw new Error(`Unsupported Pytxo release platform: ${platform} ${arch}`);
  }
  return asset;
}

module.exports = { getAssetName };
