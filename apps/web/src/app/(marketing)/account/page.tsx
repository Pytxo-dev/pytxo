import type { Metadata } from "next";
import Link from "next/link";
import { Suspense } from "react";

import { AccountAuth } from "@/components/account-auth";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export const metadata: Metadata = {
  title: "Account",
  description: "Manage your Pytxo subscription and team settings.",
};

export default function AccountPage() {
  return (
    <div className="mx-auto max-w-2xl px-4 py-20 sm:px-6">
      <h1 className="text-3xl font-semibold tracking-tight">
        <span className="chroma-text">Account</span>
      </h1>
      <p className="mt-2 text-muted-foreground">
        Pro, Max, and Ultra tiers sync entitlements through Pytxo Link after sign-in.
      </p>

      <Card className="glass-panel chroma-edge-top mt-10 border-white/8">
        <CardHeader>
          <CardTitle>Subscription</CardTitle>
          <CardDescription>Sign in to view your plan and org policies.</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <Suspense fallback={<p className="text-sm text-muted-foreground">Loading account…</p>}>
            <AccountAuth />
          </Suspense>
          <Link href="/plans" className="text-sm text-primary hover:underline">
            Compare plans →
          </Link>
        </CardContent>
      </Card>
    </div>
  );
}
