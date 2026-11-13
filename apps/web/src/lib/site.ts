/** Public distribution repo (binaries + install scripts). */
export const DISTRIBUTION_REPO = "Pytxo-dev/pytxo-releases";

/**
 * Latest published GitHub/npm tag. Download URLs, install commands, and
 * "current release" copy must use this — never the unpublished workspace
 * candidate.
 */
export const PUBLISHED_VERSION = "2.0.2";
/** In-repo / changelog candidate. Not for public download URLs. */
export const CANDIDATE_VERSION = "2.0.2";
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
export const PRODUCT_LINE = "Clear your agent PR pile in one command.";
export const PRODUCT_SUMMARY =
  "Pytxo merges your Claude Code, Codex and Copilot branches in a throwaway worktree, runs your tests, lands the ones that pass and gives each of the rest the reason and a fix prompt. Free and open source.";

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
  alt: "Clear your agent PR pile in one command. npx pytxo demo prints LANDED: 2 of 4 branches are on main now. The two left out are named with the reason: one renamed price to unitPrice and the other still uses price.",
};

/** Where a newcomer starts, linked from the plans and the 404 page. */
export const GET_STARTED_HREF = "/docs/getting-started/quickstart";

/** The header's links, beside the demo command's copy button; the docs nav shows them too, with Support. */
export const NAV_LINKS: NavLink[] = [
  { href: "/docs", label: "Docs" },
  { href: SOURCE_URL, label: "GitHub" },
];

// Display copy only. Pro is not on sale: the Action's licensing check ships
// switched off, and no page links to a checkout for it.
export const PRO_LAUNCH = "$29 once for the first 500 buyers, then $5/month or $48/year";
