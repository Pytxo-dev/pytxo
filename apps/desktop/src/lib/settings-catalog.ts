import type { SettingsSectionId } from "./navigation.svelte";

export const SETTINGS_GROUPS = ["Preferences", "Workspace", "More"] as const;

export type SettingsSection = {
  id: SettingsSectionId;
  label: string;
  group: (typeof SETTINGS_GROUPS)[number];
  description: string;
  keywords: readonly string[];
};

/** Shared destinations and search terms for Setup and the command palette. */
export const SETTINGS_SECTIONS: readonly SettingsSection[] = [
  {
    id: "general",
    label: "General",
    group: "Preferences",
    description: "Window behavior and reduced motion.",
    keywords: ["close to tray", "quit", "window", "reduced motion", "animations", "transitions"],
  },
  {
    id: "appearance",
    label: "Appearance",
    group: "Preferences",
    description: "Theme, accent, text size, and spacing.",
    keywords: ["theme", "void", "dark", "light", "match system", "accent", "color", "colour", "scale", "text size", "font size", "density", "compact", "comfortable", "display"],
  },
  {
    id: "keyboard",
    label: "Keyboard",
    group: "Preferences",
    description: "Shortcuts for Work, Approvals, and the command palette.",
    keywords: ["shortcuts", "hotkeys", "keys", "command palette", "search", "approvals", "focus work"],
  },
  {
    id: "workspaces",
    label: "Workspaces",
    group: "Workspace",
    description: "Project folders, workspace permissions, and launch defaults.",
    keywords: ["folders", "projects", "catalog", "add workspace", "open folder", "trust", "permissions", "on launch", "startup", "last used", "active workspace"],
  },
  {
    id: "agents",
    label: "Agents & permissions",
    group: "Workspace",
    description: "Installed coding agents, vendor sign-in, and default permissions.",
    keywords: ["agent cli", "codex", "claude", "cursor", "opencode", "gemini", "copilot", "aider", "antigravity", "install", "sign in", "login", "subscription", "permission profile", "deepspace", "orbit", "galaxy", "supernova", "mcp"],
  },
  {
    id: "providers",
    label: "Providers",
    group: "Workspace",
    description: "Direct API keys and custom endpoints. Agent sign-in is under Agents.",
    keywords: ["api", "keys", "byok", "environment variables", "deepseek", "openrouter", "openai", "anthropic", "custom endpoints", "models"],
  },
  {
    id: "voice",
    label: "Voice",
    group: "More",
    description: "Local Whisper, recording controls, and cloud consent information.",
    keywords: ["whisper", "model", "base.en", "microphone", "audio", "transcription", "capture", "recording", "press and hold", "click to record", "cloud fallback", "consent"],
  },
  {
    id: "privacy",
    label: "Privacy",
    group: "More",
    description: "Audio retention and transcript sanitization.",
    keywords: ["sovereign shield", "audio retention", "storage", "transcript", "sanitization", "data"],
  },
  {
    id: "account",
    label: "Account & billing",
    group: "More",
    description: "Pytxo account, plan, updates, and onboarding.",
    keywords: ["account", "billing", "tier", "plan", "payment", "invoices", "sign out", "updates", "version", "restart", "pytxo cli", "onboarding", "welcome", "community", "discord"],
  },
];

export function settingsSectionMatches(section: SettingsSection, query: string): boolean {
  const terms = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const searchable = [section.id, section.label, section.description, ...section.keywords].join(" ").toLowerCase();
  return terms.every((term) => searchable.includes(term));
}
