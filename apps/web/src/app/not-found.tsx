import type { Metadata } from "next";
import Link from "next/link";

import { SiteFooter } from "@/components/site/footer";
import { SiteHeader } from "@/components/site/header";
import s from "@/components/site/site.module.css";
import { GET_STARTED_HREF } from "@/lib/site";
import { cn } from "@/lib/utils";

// Next adds the noindex robots tag to every 404 response itself.
export const metadata: Metadata = {
  title: "Page not found",
};

export default function NotFound() {
  return (
    <div className="chassis-shell dark flex min-h-dvh flex-col bg-background text-foreground" data-chroma-theme="void" data-site="v2">
      <SiteHeader />
      <main id="main" className="w-full max-w-full flex-1 overflow-x-hidden">
        <div className={s.page}>
          <div className={cn(s.wrap, s.inner)}>
            <p className={s.ver}>404</p>
            <h1 className={s.lost}>This page does not exist.</h1>
            <p className={s.lede}>
              The link may be old, or the page moved into the docs. Start from the Quickstart or the docs.
            </p>
            <div className={s.act}>
              <Link href="/docs" className={cn(s.btn, "touch-target relative")}>Go to the docs</Link>
              <Link href={GET_STARTED_HREF} className={cn(s.quiet, "touch-target relative")}>Quickstart</Link>
              <Link href="/" className={cn(s.quiet, "touch-target relative")}>Home</Link>
            </div>
          </div>
        </div>
      </main>
      <SiteFooter />
    </div>
  );
}
