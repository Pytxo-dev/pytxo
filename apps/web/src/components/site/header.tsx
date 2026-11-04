import Link from "next/link";

import { HeaderAuth } from "@/components/header-auth";
import { CopyCommand } from "@/components/site/copy-command";
import { Mark } from "@/components/site/mark";
import s from "@/components/site/site.module.css";
import { NAV_LINKS } from "@/lib/site";

export function SiteHeader() {
  return (
    <header data-site-header className={s.nav}>
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-3 focus:z-10 focus:bg-[var(--paper)] focus:px-3 focus:py-2 focus:text-sm focus:font-medium focus:text-[var(--site-ink)]"
      >
        Skip to content
      </a>
      <div className={s.wrap}>
        <Link href="/" className={s.brand} aria-label="pytxo home">
          <Mark id="nav" />
          <span>pytxo</span>
        </Link>
        <nav className={s.links} aria-label="Main">
          {NAV_LINKS.map((link) => (
            <Link key={link.href} href={link.href} className="touch-target relative">
              {link.label}
            </Link>
          ))}
          <HeaderAuth />
          <CopyCommand variant="nav" />
        </nav>
      </div>
    </header>
  );
}
