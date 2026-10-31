import type { Metadata } from "next";
import Link from "next/link";
import { Suspense } from "react";

import { AccountAuth } from "@/components/account-auth";
import s from "@/components/site/site.module.css";
import { SITE_AUTH_ENABLED } from "@/lib/auth-config";
import { cn } from "@/lib/utils";

export const metadata: Metadata = {
  title: "Account",
  description: "Your optional Pytxo account: perks and shared receipts.",
  robots: { index: false, follow: false },
};

const QUICK_LINKS = [
  { href: "/plans", label: "Plans", detail: "Free, Pro and Team compared" },
  { href: "/bench", label: "Pybench", detail: "Published benchmark results" },
  { href: "/download", label: "Download", detail: "CLI and Desktop installers" },
  { href: "/docs", label: "Docs", detail: "Install, tutorials, reference" },
  { href: "/docs/getting-started/mcp-from-cursor", label: "MCP setup", detail: "Connect Cursor or VS Code" },
] as const;

export default function AccountPage() {
  return (
    <div className={s.page}>
      <div className={cn(s.wrap, s.inner)}>
        <h1>Account</h1>
        <p className={s.lede}>
          {SITE_AUTH_ENABLED
            ? "Pytxo runs on your machine without an account. Signing in adds hosted perks: shareable receipts, receipt history and the Pybench board."
            : "Pytxo runs on your machine without an account. Account features will appear here when sign-in is available."}
        </p>

        <section className={s.part}>
          <h2>Your account</h2>
          <p className={s.note}>Perks, connected tools and shared receipts.</p>
          <div className={s.panel}>
            <Suspense fallback={<p className={s.dim}>Loading account…</p>}>
              <AccountAuth />
            </Suspense>
          </div>
        </section>

        <section className={s.part}>
          <h2>Quick links</h2>
          <ul className={cn(s.rows, s.grid2)}>
            {QUICK_LINKS.map((item) => (
              <li key={item.href}>
                <Link href={item.href} className="touch-target relative">{item.label}</Link>
                <span className={s.dim}>{item.detail}</span>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </div>
  );
}
