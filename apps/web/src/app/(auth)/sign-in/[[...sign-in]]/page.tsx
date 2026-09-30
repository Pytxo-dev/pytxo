"use client";

import { SignIn } from "@clerk/nextjs";
import Link from "next/link";
import { useSearchParams } from "next/navigation";

import { Button } from "@/components/ui/button";
import { SITE_AUTH_ENABLED } from "@/lib/auth-config";
import { desktopAccountUrl, desktopBridgeParams, desktopSignUpUrl } from "@/lib/desktop-bridge";

export default function SignInPage() {
  const searchParams = useSearchParams();
  const bridge = desktopBridgeParams(searchParams);
  const redirectUrl = desktopAccountUrl(bridge);

  if (!SITE_AUTH_ENABLED) {
    return (
      <div className="max-w-md rounded-xl border border-border bg-card/30 p-6">
        <h1 className="text-xl font-semibold">Account sync is not configured</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          Pytxo Core and Desktop work without a Pytxo account. Agent providers
          continue to use their own official CLI sessions.
        </p>
        <Button className="mt-5" asChild>
          <Link href="/download">Download Pytxo</Link>
        </Button>
      </div>
    );
  }

  return (
    <SignIn
      routing="path"
      path="/sign-in"
      signUpUrl={desktopSignUpUrl(bridge)}
      forceRedirectUrl={redirectUrl}
      fallbackRedirectUrl={redirectUrl}
    />
  );
}
