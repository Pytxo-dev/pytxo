import Link from "next/link";

import { Separator } from "@/components/ui/separator";
import { GITHUB_URL, NAV_LINKS } from "@/lib/site";

export function SiteFooter() {
  return (
    <footer className="mt-auto border-t border-white/5">
      <div className="header-chroma-line w-full opacity-60" aria-hidden />
      <div className="mx-auto flex max-w-6xl flex-col gap-6 px-4 py-12 sm:px-6">
        <div className="flex flex-col gap-6 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex flex-col gap-1.5">
            <p className="font-semibold tracking-tight">Pytxo</p>
            <p className="max-w-sm text-sm text-muted-foreground">
              Agent hypervisor and telemetry plane. Open-source Rust core.
            </p>
          </div>
          <nav className="flex flex-wrap gap-x-5 gap-y-2 text-sm text-muted-foreground">
            {NAV_LINKS.map((link) => (
              <Link key={link.href} href={link.href} className="hover:text-foreground">
                {link.label}
                {link.badge ? ` (${link.badge})` : ""}
              </Link>
            ))}
            <a
              href={GITHUB_URL}
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-foreground"
            >
              GitHub
            </a>
          </nav>
        </div>
        <Separator className="bg-white/8" />
        <p className="text-xs text-muted-foreground">
          © {new Date().getFullYear()} Pytxo. MIT License.{" "}
          <a href="https://pytxo.com" className="hover:text-foreground">
            pytxo.com
          </a>
        </p>
      </div>
    </footer>
  );
}
