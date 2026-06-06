"use strict";

const { platform, arch } = process;

function getAssetName() {
  if (platform === "win32") {
    if (arch === "arm64") return "pytxo-windows-arm64.exe";
    return "pytxo-windows-x64.exe";
  }
  if (platform === "darwin") {
    if (arch === "arm64") return "pytxo-darwin-arm64";
    return "pytxo-darwin-x64";
  }
  if (platform === "linux") {
    if (arch === "arm64") return "pytxo-linux-arm64";
    return "pytxo-linux-x64";
  }
  throw new Error(`Unsupported platform: ${platform} ${arch}`);
}

module.exports = { getAssetName };
