#!/usr/bin/env bash
# Install pytxo CLI from GitHub Releases into ~/.local/bin (or PYTXO_INSTALL_DIR).
set -euo pipefail

REPO="${PYTXO_REPO:-Pytxo-dev/pytxo-releases}"
INSTALL_DIR="${PYTXO_INSTALL_DIR:-$HOME/.local/bin}"

detect_asset() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"
  case "$os" in
    linux)
      case "$arch" in
        x86_64|amd64) echo "pytxo-linux-x64" ;;
        aarch64|arm64) echo "pytxo-linux-arm64" ;;
        *) echo "unsupported linux arch: $arch" >&2; exit 1 ;;
      esac
      ;;
    darwin)
      case "$arch" in
        x86_64) echo "pytxo-darwin-x64" ;;
        arm64|aarch64) echo "pytxo-darwin-arm64" ;;
        *) echo "unsupported macOS arch: $arch" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "unsupported OS: $os (use: npm i -g pytxo)" >&2
      exit 1
      ;;
  esac
}

resolve_version() {
  if [[ -n "${PYTXO_VERSION:-}" ]]; then
    echo "$PYTXO_VERSION"
    return
  fi
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' \
    | head -n1
}

VERSION="$(resolve_version)"
ASSET="$(detect_asset)"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"

mkdir -p "$INSTALL_DIR"
TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

echo "Installing pytxo ${VERSION} (${ASSET}) → ${INSTALL_DIR}/pytxo"
curl -fsSL "$URL" -o "$TMP"
chmod +x "$TMP"
mv "$TMP" "${INSTALL_DIR}/pytxo"

if ! echo ":$PATH:" | grep -q ":${INSTALL_DIR}:"; then
  echo ""
  echo "Add to PATH:  export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

"${INSTALL_DIR}/pytxo" doctor || true
echo "Done."
