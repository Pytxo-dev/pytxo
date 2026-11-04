import Link from "next/link";

import { Mark } from "@/components/site/mark";
import s from "@/components/site/site.module.css";
import { DISCORD_URL, SOURCE_URL } from "@/lib/site";

const LINKS = [
  { href: "/docs", label: "Docs" },
  { href: SOURCE_URL, label: "GitHub" },
  { href: "/docs/evidence", label: "Evidence" },
  { href: "/bench", label: "Pybench" },
  { href: "/plans", label: "Plans" },
  { href: "/download", label: "Desktop (frozen)" },
  { href: DISCORD_URL, label: "Discord" },
] as const;

const LEGAL_LINKS = [
  { href: "/terms", label: "Terms" },
  { href: "/privacy", label: "Privacy" },
  { href: "/refunds", label: "Refunds" },
] as const;

export function SiteFooter() {
  return (
    <footer className={s.foot}>
      <div className={s.wrap}>
        <div className={s.row}>
          <Link href="/" className={s.brand} aria-label="pytxo home">
            <Mark id="foot" />
            <span>pytxo</span>
          </Link>
          <nav aria-label="Footer">
            {LINKS.map((link) => (
              <Link key={link.href} href={link.href} className="touch-target relative">
                {link.label}
              </Link>
            ))}
          </nav>
        </div>
        <div className={s.row}>
          <p>MIT. Free on your machine and on public repositories.</p>
          <nav aria-label="Legal">
            {LEGAL_LINKS.map((link) => (
              <Link key={link.href} href={link.href} className="touch-target relative">
                {link.label}
              </Link>
            ))}
          </nav>
        </div>
      </div>
    </footer>
  );
}
