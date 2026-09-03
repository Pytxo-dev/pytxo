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
if [[ -z "$VERSION" ]]; then
  echo "could not resolve a Pytxo release version" >&2
  exit 1
fi
if [[ "$VERSION" != v* ]]; then VERSION="v${VERSION}"; fi
EXPECTED_VERSION="${VERSION#v}"
ASSET="$(detect_asset)"
RELEASE_BASE="${PYTXO_RELEASE_BASE_URL:-https://github.com/${REPO}/releases/download}"
RELEASE_BASE="${RELEASE_BASE%/}"
URL="${RELEASE_BASE}/${VERSION}/${ASSET}"
CHECKSUM_URL="${RELEASE_BASE}/${VERSION}/SHA256SUMS.txt"

mkdir -p "$INSTALL_DIR"
TMP="$(mktemp "${INSTALL_DIR}/.pytxo-download.XXXXXX")"
SUMS="$(mktemp "${INSTALL_DIR}/.pytxo-checksums.XXXXXX")"
trap 'rm -f "$TMP" "$SUMS"' EXIT

echo "Installing pytxo ${VERSION} (${ASSET}) → ${INSTALL_DIR}/pytxo"
curl -fsSL "$CHECKSUM_URL" -o "$SUMS"
curl -fsSL "$URL" -o "$TMP"

EXPECTED_SHA="$(awk -v asset="$ASSET" '$2 == asset || $2 == "*" asset { print tolower($1); exit }' "$SUMS")"
if [[ ! "$EXPECTED_SHA" =~ ^[0-9a-f]{64}$ ]]; then
  echo "SHA256SUMS.txt has no exact entry for ${ASSET}" >&2
  exit 1
fi
if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL_SHA="$(sha256sum "$TMP" | awk '{print tolower($1)}')"
elif command -v shasum >/dev/null 2>&1; then
  ACTUAL_SHA="$(shasum -a 256 "$TMP" | awk '{print tolower($1)}')"
else
  echo "sha256sum or shasum is required to verify Pytxo" >&2
  exit 1
fi
if [[ "$ACTUAL_SHA" != "$EXPECTED_SHA" ]]; then
  echo "SHA-256 mismatch for ${ASSET}" >&2
  exit 1
fi
chmod +x "$TMP"
VERSION_OUTPUT="$("$TMP" --version)"
if [[ "$VERSION_OUTPUT" != "pytxo ${EXPECTED_VERSION}" && "$VERSION_OUTPUT" != "pytxo v${EXPECTED_VERSION}" ]]; then
  echo "downloaded binary reported unexpected version: ${VERSION_OUTPUT}" >&2
  exit 1
fi
mv "$TMP" "${INSTALL_DIR}/pytxo"

if ! echo ":$PATH:" | grep -q ":${INSTALL_DIR}:"; then
  echo ""
  echo "Add to PATH:  export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

echo "Installed $(${INSTALL_DIR}/pytxo --version). Run 'pytxo doctor' inside a repository."
