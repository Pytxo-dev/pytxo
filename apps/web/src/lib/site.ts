/** Public distribution repo (binaries + install scripts). Main pytxo monorepo is private. */
export const DISTRIBUTION_REPO = "Pytxo-dev/pytxo-releases";
export const PYTXO_VERSION = "0.3.0";

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
  { href: "/plans", label: "Plans", badge: "Soon" },
  { href: "/download", label: "Download" },
];
