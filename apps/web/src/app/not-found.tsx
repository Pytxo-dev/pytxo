import type { Metadata } from "next";
import Link from "next/link";

import { SiteFooter } from "@/components/site/footer";
import { SiteHeader } from "@/components/site/header";
import { Button } from "@/components/ui/button";

// Next adds the noindex robots tag to every 404 response itself.
export const metadata: Metadata = {
  title: "Page not found",
};

export default function NotFound() {
  return (
    <div className="chassis-shell dark min-h-dvh bg-background text-foreground" data-chroma-theme="void">
      <SiteHeader />
      <main id="main" className="marketing-scroll w-full max-w-full overflow-x-hidden">
        <div className="mx-auto flex max-w-3xl flex-col gap-4 px-4 py-24 sm:px-6 sm:py-32">
          <p className="font-mono text-sm text-muted-foreground">404</p>
          <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">This page does not exist.</h1>
          <p className="max-w-2xl text-lg text-muted-foreground">
            The link may be old, or the page moved into the docs. Start from the docs or the download page.
          </p>
          <div className="mt-2 flex flex-wrap gap-3">
            <Button size="lg" className="touch-target relative" asChild>
              <Link href="/docs">Go to the docs</Link>
            </Button>
            <Button size="lg" variant="outline" className="touch-target relative" asChild>
              <Link href="/download">Download Pytxo</Link>
            </Button>
            <Button size="lg" variant="outline" className="touch-target relative" asChild>
              <Link href="/">Home</Link>
            </Button>
          </div>
        </div>
        <SiteFooter />
      </main>
    </div>
  );
}
