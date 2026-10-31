import type { Metadata } from "next";
import { Suspense } from "react";

import { AccountConnect } from "@/components/account-connect";
import s from "@/components/site/site.module.css";
import { ACCOUNT_PERKS } from "@/lib/account-connect";
import { cn } from "@/lib/utils";

export const metadata: Metadata = {
  title: "Connect Pytxo",
  description: "Connect the Pytxo CLI or Desktop to your optional Pytxo account.",
  robots: { index: false, follow: false },
};

export default function AccountConnectPage() {
  return <div className={s.page}>
    <div className={cn(s.wrap, s.inner)}>
      <h1>Connect Pytxo</h1>
      <p className={s.lede}>Optional. Everything local works without an account; signing in adds a few hosted perks.</p>
      <div className={s.panel}>
        <Suspense fallback={<p className={s.dim}>Loading…</p>}>
          <AccountConnect />
        </Suspense>
      </div>

      <section className={s.part}>
        <h2>What signing in adds</h2>
        <dl className={cn(s.facts, s.stack)}>
          {ACCOUNT_PERKS.map(([term, detail]) => <div key={term}>
            <dt>{term}</dt>
            <dd>{detail}</dd>
          </div>)}
        </dl>
      </section>
    </div>
  </div>;
}
