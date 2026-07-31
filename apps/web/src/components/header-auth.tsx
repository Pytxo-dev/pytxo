"use client";

import Link from "next/link";
import { SignedIn, SignedOut, SignUpButton, UserButton } from "@clerk/nextjs";

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
          className="hidden text-muted-foreground sm:inline-flex"
          asChild
        >
          <Link href="/sign-in">Sign in</Link>
        </Button>
        <SignUpButton mode="redirect" forceRedirectUrl="/account">
          <Button size="sm" variant="outline" className="hidden border-border bg-transparent sm:inline-flex">
            Sign up
          </Button>
        </SignUpButton>
      </SignedOut>
      <SignedIn>
        <Button variant="ghost" size="sm" className="hidden sm:inline-flex" asChild>
          <Link href="/account">Account</Link>
        </Button>
        <UserButton afterSignOutUrl="/" />
      </SignedIn>
    </div>
  );
}
