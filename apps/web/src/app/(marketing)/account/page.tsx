import type { Metadata } from "next";
import Link from "next/link";
import { Suspense } from "react";

import { AccountAuth } from "@/components/account-auth";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { SITE_AUTH_ENABLED } from "@/lib/auth-config";

export const metadata: Metadata = {
  title: "Account",
  description: "View account access for a configured Pytxo deployment.",
  robots: { index: false, follow: false },
};

const QUICK_LINKS = [
  { href: "/plans", label: "Availability", detail: "Local Core and configured Cloud / Teams" },
  { href: "/download", label: "Download", detail: "CLI and Desktop installers" },
  { href: "/docs", label: "Docs", detail: "Install, tutorials, reference" },
  { href: "/docs/getting-started/mcp-from-cursor", label: "MCP setup", detail: "Connect Cursor or VS Code" },
] as const;

export default function AccountPage() {
  return (
    <div className="mx-auto max-w-4xl px-4 py-16 sm:px-6 sm:py-24">
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-semibold tracking-tight sm:text-4xl">Account</h1>
        <p className="max-w-2xl text-muted-foreground">
          {SITE_AUTH_ENABLED
            ? "Sign in to view entitlements provided by this deployment. Local Core works without an account."
            : "Local Core works without an account. Account and paid-entitlement sync will appear here when production authentication is configured."}
        </p>
      </div>

      <div className="mt-10 grid gap-8 lg:grid-cols-[1.2fr_0.8fr] lg:gap-12">
        <Card className="border-border bg-card/30">
          <CardHeader>
            <CardTitle>Account access</CardTitle>
            <CardDescription>Deployment-provided entitlements, agent limits, and organization policy.</CardDescription>
          </CardHeader>
          <CardContent>
            <Suspense fallback={<p className="text-sm text-muted-foreground">Loading account…</p>}>
              <AccountAuth />
            </Suspense>
          </CardContent>
        </Card>

        <div className="flex flex-col gap-4 border-t border-border pt-6 lg:border-t-0 lg:border-l lg:pl-8 lg:pt-0">
          <div>
            <h2 className="text-sm font-semibold tracking-tight">Quick links</h2>
            <p className="mt-1 text-xs text-muted-foreground">Common next steps from your account.</p>
          </div>
          <nav className="flex flex-col gap-1">
            {QUICK_LINKS.map((item) => (
              <Link
                key={item.href}
                href={item.href}
                className="rounded-[var(--radius-md)] px-2 py-2.5 transition-colors hover:bg-muted/40"
              >
                <span className="block text-sm font-medium text-foreground">{item.label}</span>
                <span className="mt-0.5 block text-xs text-muted-foreground">{item.detail}</span>
              </Link>
            ))}
          </nav>
        </div>
      </div>
    </div>
  );
}
