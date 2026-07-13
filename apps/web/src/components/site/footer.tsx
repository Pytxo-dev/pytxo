import Image from "next/image";
import Link from "next/link";

import { GITHUB_URL } from "@/lib/site";

const PRODUCT_LINKS = [
  { href: "/download", label: "Download" },
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
  { href: "/docs/reference/cli", label: "CLI reference" },
] as const;

export function SiteFooter() {
  return (
    <footer className="mt-auto border-t border-border">
      <div className="mx-auto max-w-6xl px-4 py-12 sm:px-6">
        <div className="flex flex-col gap-10 lg:flex-row lg:justify-between">
          <div className="flex max-w-sm flex-col gap-3">
            <div className="flex items-center gap-2.5">
              <Image src="/logo.png" alt="" width={28} height={28} className="size-7" />
              <p className="font-semibold tracking-tight">Pytxo</p>
            </div>
            <p className="text-sm leading-relaxed text-muted-foreground">
              Local agent coordinator. Open-source Rust core. Optional Desktop for topology and
              approvals.
            </p>
          </div>

          <div className="grid grid-cols-3 gap-8 text-sm">
            <div>
              <p className="font-medium text-foreground">Product</p>
              <ul className="mt-3 flex flex-col gap-2 text-muted-foreground">
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
              <ul className="mt-3 flex flex-col gap-2 text-muted-foreground">
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
              <ul className="mt-3 flex flex-col gap-2 text-muted-foreground">
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

        <div className="mt-10 border-t border-border pt-6">
          <p className="text-xs text-muted-foreground">
            © {new Date().getFullYear()} Pytxo. MIT License.
          </p>
        </div>
      </div>
    </footer>
  );
}
