/** Public distribution repo (binaries + install scripts). */
export const DISTRIBUTION_REPO = "Pytxo-dev/pytxo-releases";

/**
 * Latest published GitHub/npm tag. Download URLs, install commands, and
 * "current release" copy must use this — never the unpublished workspace
 * candidate.
 */
export const PUBLISHED_VERSION = "1.3.6";
/** In-repo / changelog candidate. Not for public download URLs. */
export const CANDIDATE_VERSION = "2.0.0";
/** Alias for published tag used by download UI. */
export const PYTXO_VERSION = PUBLISHED_VERSION;

export const DESKTOP_RELEASE_BASE =
  `https://github.com/${DISTRIBUTION_REPO}/releases/download/v${PUBLISHED_VERSION}`;

/** Display name for the optional desktop app. */
export const DESKTOP_PRODUCT_NAME = "Pytxo Desktop";

/** Installer asset URLs on the public distribution repo (v0.5.0+ desktop naming). */
export const DESKTOP_DOWNLOADS = {
  windows: `${DESKTOP_RELEASE_BASE}/pytxo-desktop-windows-x64.msi`,
  macosArm: `${DESKTOP_RELEASE_BASE}/pytxo-desktop-darwin-arm64.dmg`,
  macosX64: `${DESKTOP_RELEASE_BASE}/pytxo-desktop-darwin-x64.dmg`,
  linux: `${DESKTOP_RELEASE_BASE}/pytxo-desktop-linux-x64.AppImage`,
} as const;

export const GITHUB_URL = `https://github.com/${DISTRIBUTION_REPO}`;
export const RELEASES_URL = `${GITHUB_URL}/releases`;
/** MIT-licensed product source. */
export const SOURCE_URL = "https://github.com/Pytxo-dev/pytxo";
/** Community Discord invite (canonical). */
export const DISCORD_URL = "https://discord.gg/AUFRPFjSYv";
export const NPM_URL = "https://www.npmjs.com/package/pytxo";
export const NPM_INSTALL = `npm i -g pytxo@${PUBLISHED_VERSION}`;
/** The one command the site leads with: no install, no flags. */
export const DEMO_COMMAND = "npx pytxo demo";
/** The GitHub Action, on its floating major tag. */
export const ACTION_USES = "Pytxo-dev/pytxo/actions/land@v2";

export const INSTALL_SH_URL = `https://raw.githubusercontent.com/${DISTRIBUTION_REPO}/main/install.sh`;
export const INSTALL_PS1_URL = `https://raw.githubusercontent.com/${DISTRIBUTION_REPO}/main/install.ps1`;

export const INSTALL_SH_CMD = `curl -fsSL ${INSTALL_SH_URL} | bash`;
export const INSTALL_PS1_CMD = `irm ${INSTALL_PS1_URL} | iex`;

export type NavLink = {
  href: string;
  label: string;
  badge?: string;
};

/** One product sentence, used by metadata, footer and structured data. */
export const PRODUCT_LINE = "Each agent's branch passes. Do they pass together?";
export const PRODUCT_SUMMARY =
  "Pytxo merges your Claude Code, Codex and Copilot branches in a throwaway worktree, runs your tests on the combined result, and blocks the merge if it breaks. Free, open source, runs on your machine or in CI.";

/** Agents verified live on a released build vs. those still in beta. */
export const AGENT_SUPPORT = {
  verified: ["Codex", "Claude Code", "Cursor Agent"],
  beta: ["OpenCode", "Antigravity"],
} as const;

/** Every page's share image: the demo's verdict, from the recorded output. */
export const SHARE_IMAGE = {
  url: "/social/check-1600x1000.png",
  width: 1600,
  height: 1000,
  alt: "Each agent's branch passes. Do they pass together? npx pytxo demo prints BLOCKED: two branches pass npm test alone but fail together, because one renamed price to unitPrice and the other still uses price.",
};

/** The header's call to action; the header renders it as its one button. */
export const GET_STARTED_HREF = "/docs/getting-started/quickstart";

export const NAV_LINKS: NavLink[] = [
  { href: "/#how", label: "How it works" },
  { href: "/#ci", label: "In CI" },
  { href: "/bench", label: "Pybench" },
  { href: "/docs", label: "Docs" },
  { href: DISCORD_URL, label: "Support" },
  { href: GET_STARTED_HREF, label: "Get started" },
];
