"use client";

import Link from "next/link";
import { SignedIn, SignedOut, UserButton } from "@clerk/nextjs";

import { SITE_AUTH_ENABLED } from "@/lib/auth-config";

// Its links sit in the header's row as they are, so a hidden one takes no gap there.
export function HeaderAuth() {
  if (!SITE_AUTH_ENABLED) {
    return null;
  }

  return (
    <div className="contents">
      <SignedOut>
        <Link href="/sign-in" className="touch-target relative hidden sm:inline">
          Sign in
        </Link>
      </SignedOut>
      <SignedIn>
        <Link href="/account" className="touch-target relative hidden sm:inline">
          Account
        </Link>
        <UserButton afterSignOutUrl="/" />
      </SignedIn>
    </div>
  );
}
