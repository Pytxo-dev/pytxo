export const GITHUB_URL = "https://github.com/Pytxo-dev/pytxo";

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
