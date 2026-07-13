/** Public distribution repo (binaries + install scripts). */
export const DISTRIBUTION_REPO = "Pytxo-dev/pytxo-releases";
export const PYTXO_VERSION = "0.4.0";

export const DESKTOP_RELEASE_BASE =
  `https://github.com/${DISTRIBUTION_REPO}/releases/download/v${PYTXO_VERSION}`;

/** Display name for the optional desktop app. */
export const DESKTOP_PRODUCT_NAME = "Pytxo Desktop";

/**
 * Installer asset URLs still use the legacy `pytxo-reality-deck-*` filenames until
 * a release renames artifacts. Labels in the UI use DESKTOP_PRODUCT_NAME.
 */
export const DESKTOP_DOWNLOADS = {
  windows: `${DESKTOP_RELEASE_BASE}/pytxo-reality-deck-windows-x64.msi`,
  macosArm: `${DESKTOP_RELEASE_BASE}/pytxo-reality-deck-darwin-arm64.dmg`,
  macosX64: `${DESKTOP_RELEASE_BASE}/pytxo-reality-deck-darwin-x64.dmg`,
  linux: `${DESKTOP_RELEASE_BASE}/pytxo-reality-deck-linux-x64.AppImage`,
} as const;

export const GITHUB_URL = `https://github.com/${DISTRIBUTION_REPO}`;
export const RELEASES_URL = `${GITHUB_URL}/releases`;
export const NPM_URL = "https://www.npmjs.com/package/pytxo";
export const NPM_INSTALL = "npm i -g pytxo";

export const INSTALL_SH_URL = `https://raw.githubusercontent.com/${DISTRIBUTION_REPO}/main/install.sh`;
export const INSTALL_PS1_URL = `https://raw.githubusercontent.com/${DISTRIBUTION_REPO}/main/install.ps1`;

export const INSTALL_SH_CMD = `curl -fsSL ${INSTALL_SH_URL} | bash`;
export const INSTALL_PS1_CMD = `irm ${INSTALL_PS1_URL} | iex`;

export type NavLink = {
  href: string;
  label: string;
  badge?: string;
};

export const NAV_LINKS: NavLink[] = [
  { href: "/", label: "Home" },
  { href: "/docs", label: "Docs" },
  { href: "/plans", label: "Plans" },
  { href: "/download", label: "Download" },
];
