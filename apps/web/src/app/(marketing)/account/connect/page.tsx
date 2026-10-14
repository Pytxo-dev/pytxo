import type { Metadata } from "next";
import { Suspense } from "react";

import { AccountConnect } from "@/components/account-connect";
import { ACCOUNT_PERKS } from "@/lib/account-connect";

export const metadata: Metadata = {
  title: "Connect Pytxo",
  description: "Connect the Pytxo CLI or Desktop to your optional Pytxo account.",
  robots: { index: false, follow: false },
};

export default function AccountConnectPage() {
  return <div className="mx-auto grid max-w-5xl gap-12 px-4 py-16 sm:px-6 sm:py-24 lg:grid-cols-[1.1fr_0.9fr]">
    <div>
      <h1 className="text-3xl font-semibold tracking-tight sm:text-4xl">Connect Pytxo</h1>
      <p className="mt-3 max-w-xl text-muted-foreground">Optional. Everything local works without an account; signing in adds a few hosted perks.</p>
      <div className="mt-8 rounded-md border border-white/15 p-6">
        <Suspense fallback={<p className="text-sm text-muted-foreground">Loading…</p>}>
          <AccountConnect />
        </Suspense>
      </div>
    </div>
    <div>
      <h2 className="text-sm font-semibold tracking-tight">What signing in adds</h2>
      <dl className="mt-4 divide-y divide-white/10 border-y border-white/10">
        {ACCOUNT_PERKS.map(([term, detail]) => <div key={term} className="py-4">
          <dt className="text-sm">{term}</dt>
          <dd className="mt-1 text-sm leading-relaxed text-muted-foreground">{detail}</dd>
        </div>)}
      </dl>
    </div>
  </div>;
}
