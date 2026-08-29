import Image from "next/image";
import Link from "next/link";

import { DISCORD_URL, GITHUB_URL } from "@/lib/site";

const PRODUCT_LINKS = [
  { href: "/download", label: "Download" },
  { href: "/evidence", label: "Evidence" },
  { href: "/plans", label: "Plans" },
  { href: "/account", label: "Account" },
] as const;

const DOCS_LINKS = [
  { href: "/docs", label: "Docs home" },
  { href: "/docs/getting-started/install", label: "Install" },
  { href: "/docs/concepts/desktop", label: "Pytxo Desktop" },
] as const;

const RESOURCE_LINKS = [
  { href: GITHUB_URL, label: "GitHub", external: true },
  { href: DISCORD_URL, label: "Discord", external: true },
  { href: "/docs/reference/cli", label: "CLI reference" },
] as const;

export function SiteFooter() {
  return (
    <footer className="mt-auto border-t border-white/[0.08] bg-[#050507]">
      <div className="mx-auto max-w-[92rem] px-4 py-14 sm:px-6 lg:px-10">
        <div className="flex flex-col gap-10 lg:flex-row lg:justify-between">
          <div className="flex max-w-sm flex-col gap-3">
            <div className="flex items-center gap-2.5">
              <Image src="/logo-mark.png" alt="" width={28} height={28} className="size-7" />
              <p className="font-semibold tracking-tight">Pytxo</p>
            </div>
            <p className="text-sm leading-relaxed text-[#7d7d87]">
              The local commit layer for coding agents. Open-source Rust core with optional Desktop operations.
            </p>
          </div>

          <div className="grid grid-cols-2 gap-10 text-sm sm:grid-cols-3">
            <div>
              <p className="font-medium text-foreground">Product</p>
              <ul className="mt-4 flex flex-col gap-2.5 text-[#7d7d87]">
                {PRODUCT_LINKS.map((link) => (
                  <li key={link.href}>
                    <Link href={link.href} className="hover:text-foreground">
                      {link.label}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <p className="font-medium text-foreground">Docs</p>
              <ul className="mt-4 flex flex-col gap-2.5 text-[#7d7d87]">
                {DOCS_LINKS.map((link) => (
                  <li key={link.href}>
                    <Link href={link.href} className="hover:text-foreground">
                      {link.label}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <p className="font-medium text-foreground">Resources</p>
              <ul className="mt-4 flex flex-col gap-2.5 text-[#7d7d87]">
                {RESOURCE_LINKS.map((link) => (
                  <li key={link.href}>
                    {"external" in link && link.external ? (
                      <a
                        href={link.href}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="hover:text-foreground"
                      >
                        {link.label}
                      </a>
                    ) : (
                      <Link href={link.href} className="hover:text-foreground">
                        {link.label}
                      </Link>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>

        <div className="mt-14 flex items-center justify-between border-t border-white/[0.08] pt-6">
          <p className="font-mono text-[10px] text-[#66666f]">
            © {new Date().getFullYear()} Pytxo. MIT License.
          </p>
          <span className="font-mono text-[10px] text-[#66666f]">LOCAL BY DEFAULT</span>
        </div>
      </div>
    </footer>
  );
}
