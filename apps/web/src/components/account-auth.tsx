"use client";

import Link from "next/link";
import { useAuth } from "@clerk/nextjs";
import { useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";
import { SignedIn, SignedOut, SignUpButton, UserButton } from "@clerk/nextjs";

import { Button } from "@/components/ui/button";

type EntitlementPayload = {
  tier?: string;
  max_agents?: number;
  cloud_enabled?: boolean;
  error?: string;
};

export function AccountAuth() {
  const [entitlements, setEntitlements] = useState<EntitlementPayload | null>(null);
  const [loading, setLoading] = useState(false);
  const searchParams = useSearchParams();
  const deckCallback = searchParams.get("deck_callback");
  const { isSignedIn, getToken } = useAuth();

  useEffect(() => {
    let cancelled = false;
    async function load() {
      setLoading(true);
      try {
        const res = await fetch("/api/entitlements/status");
        if (!res.ok) {
          if (!cancelled) setEntitlements({ error: `HTTP ${res.status}` });
          return;
        }
        const data = (await res.json()) as EntitlementPayload;
        if (!cancelled) setEntitlements(data);
      } catch {
        if (!cancelled) setEntitlements({ error: "fetch_failed" });
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    load();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!isSignedIn || deckCallback !== "pytxo-deck") return;
    let cancelled = false;
    (async () => {
      const token = await getToken();
      if (!cancelled && token) {
        window.location.href = `pytxo-deck://auth?token=${encodeURIComponent(token)}`;
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [isSignedIn, deckCallback, getToken]);

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
        {loading && (
          <p className="text-sm text-muted-foreground">Loading entitlements…</p>
        )}
        {entitlements && !loading && (
          <div className="rounded-lg border border-white/10 bg-black/20 px-4 py-3 text-sm">
            {entitlements.error ? (
              <p className="text-muted-foreground">
                Could not reach Pytxo Link ({entitlements.error}). Local Core tier still works.
              </p>
            ) : (
              <dl className="grid grid-cols-2 gap-2">
                <dt className="text-muted-foreground">Tier</dt>
                <dd className="font-medium capitalize">{entitlements.tier ?? "core"}</dd>
                <dt className="text-muted-foreground">Max agents</dt>
                <dd>{entitlements.max_agents ?? 3}</dd>
                <dt className="text-muted-foreground">Cloud</dt>
                <dd>{entitlements.cloud_enabled ? "enabled" : "off"}</dd>
              </dl>
            )}
          </div>
        )}
        <Link
          href="https://pytxo.com/account#subscription"
          className="text-sm text-primary hover:underline"
        >
          Manage subscription in Paddle portal →
        </Link>
      </SignedIn>
    </div>
  );
}
