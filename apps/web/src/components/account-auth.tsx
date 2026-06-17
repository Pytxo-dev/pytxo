"use client";

import Link from "next/link";
import { SignedIn, SignedOut, SignUpButton, UserButton } from "@clerk/nextjs";

import { Button } from "@/components/ui/button";

export function AccountAuth() {
  return (
    <div className="flex flex-col gap-3">
      <SignedOut>
        <div className="flex flex-wrap gap-2">
          <Button asChild>
            <Link href="/sign-in">Sign in</Link>
          </Button>
          <SignUpButton mode="redirect" forceRedirectUrl="/account">
            <Button variant="outline">Sign up</Button>
          </SignUpButton>
        </div>
        <p className="text-sm text-muted-foreground">
          Core (local CLI + Deck) works without an account. Sign in to sync Pro, Max, or Ultra
          entitlements.
        </p>
      </SignedOut>
      <SignedIn>
        <div className="flex items-center gap-3">
          <UserButton afterSignOutUrl="/" />
          <span className="text-sm text-muted-foreground">
            Signed in — entitlements sync via Pytxo Link
          </span>
        </div>
      </SignedIn>
    </div>
  );
}
