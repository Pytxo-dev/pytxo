import type { ReactNode } from "react";

import s from "@/components/site/site.module.css";
import { cn } from "@/lib/utils";

/** Plain reading layout for Terms, Privacy and Refunds. */
export function LegalPage({ title, updated, children }: { title: string; updated: string; children: ReactNode }) {
  return (
    <article className={s.page}>
      <div className={cn(s.wrap, s.inner)}>
        <h1>{title}</h1>
        <p className={s.lede}>Last updated {updated}</p>
        <div className={s.prose}>{children}</div>
      </div>
    </article>
  );
}

export const LEGAL_CONTACT = "contact@mbcz.xyz";
export const LEGAL_UPDATED = "October 14, 2026";
