"use client";

import Link from "next/link";
import { SignedIn, SignedOut, UserButton } from "@clerk/nextjs";

import { Button } from "@/components/ui/button";
import { SITE_AUTH_ENABLED } from "@/lib/auth-config";

export function HeaderAuth() {
  if (!SITE_AUTH_ENABLED) {
    return null;
  }

  return (
    <div className="flex items-center gap-2">
      <SignedOut>
        <Button
          variant="ghost"
          size="sm"
          className="hidden h-8 text-[13px] text-muted-foreground hover:text-foreground sm:inline-flex"
          asChild
        >
          <Link href="/sign-in">Sign in</Link>
        </Button>
      </SignedOut>
      <SignedIn>
        <Button variant="ghost" size="sm" className="hidden h-8 text-[13px] sm:inline-flex" asChild>
          <Link href="/account">Account</Link>
        </Button>
        <UserButton afterSignOutUrl="/" />
      </SignedIn>
    </div>
  );
}
