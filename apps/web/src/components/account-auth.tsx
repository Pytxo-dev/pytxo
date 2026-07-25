"use client";

import Link from "next/link";
import { useAuth } from "@clerk/nextjs";
import { useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";
import { SignedIn, SignedOut, SignUpButton, UserButton } from "@clerk/nextjs";

import { Button } from "@/components/ui/button";

type OrgPolicy = {
  default_permission_profile?: string | null;
  shared_trusted_domains?: string[];
};

type OrgSeats = {
  seats_total?: number;
  seats_used?: number;
  seats_available?: number;
};

type EntitlementPayload = {
  tier?: string;
  max_agents?: number;
  cloud_enabled?: boolean;
  org_id?: string | null;
  org_policy?: OrgPolicy;
  org_seats?: OrgSeats;
  error?: string;
};

export function AccountAuth() {
  const [entitlements, setEntitlements] = useState<EntitlementPayload | null>(null);
  const [loading, setLoading] = useState(false);
  const [deckToken, setDeckToken] = useState<string | null>(null);
  const [deckTokenError, setDeckTokenError] = useState<string | null>(null);
  const searchParams = useSearchParams();
  const deckCallback = searchParams.get("deck_callback");
  const { isSignedIn, isLoaded, getToken } = useAuth();
  const wantsDesktopReturn = deckCallback === "pytxo-deck";

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

  // Resolve a Clerk JWT for the Desktop deep link. Prefer an explicit user-gesture
  // <a href="pytxo-deck://..."> because Chromium often blocks programmatic custom-protocol
  // redirects from an async effect.
  useEffect(() => {
    if (!isLoaded || !isSignedIn || !wantsDesktopReturn) {
      setDeckToken(null);
      setDeckTokenError(null);
      return;
    }
    let cancelled = false;
    let attempts = 0;
    async function resolveToken() {
      while (!cancelled && attempts < 8) {
        attempts += 1;
        try {
          const token = await getToken();
          if (token) {
            if (!cancelled) {
              setDeckToken(token);
              setDeckTokenError(null);
            }
            return;
          }
        } catch {
          /* retry */
        }
        await new Promise((r) => setTimeout(r, 250));
      }
      if (!cancelled) {
        setDeckTokenError("Could not get a session token. Refresh and try again.");
      }
    }
    void resolveToken();
    return () => {
      cancelled = true;
    };
  }, [isLoaded, isSignedIn, wantsDesktopReturn, getToken]);

  const deckReturnHref = deckToken
    ? `pytxo-deck://auth?token=${encodeURIComponent(deckToken)}`
    : null;

  const policy = entitlements?.org_policy;
  const seats = entitlements?.org_seats;

  return (
    <div className="flex flex-col gap-4">
      <SignedOut>
        <div className="flex flex-wrap gap-2">
          <Button asChild>
            <Link
              href={
                wantsDesktopReturn
                  ? "/sign-in?deck_callback=pytxo-deck"
                  : "/sign-in"
              }
            >
              Sign in
            </Link>
          </Button>
          <SignUpButton
            mode="redirect"
            forceRedirectUrl={
              wantsDesktopReturn
                ? "/account?deck_callback=pytxo-deck"
                : "/account"
            }
          >
            <Button variant="outline">Sign up</Button>
          </SignUpButton>
        </div>
        <p className="text-sm text-muted-foreground">
          Core (local CLI and Desktop) works without an account. Sign in to sync Pro, Max, or Ultra
          entitlements.
        </p>
        <Link href="/plans" className="text-sm text-primary hover:underline">
          Compare plans
        </Link>
      </SignedOut>
      <SignedIn>
        <div className="flex items-center gap-3">
          <UserButton afterSignOutUrl="/" />
          <span className="text-sm text-muted-foreground">
            Signed in. Entitlements sync via Pytxo Link.
          </span>
        </div>
        {wantsDesktopReturn && (
          <div className="rounded-xl border border-white/10 bg-background/50 px-4 py-4 text-sm flex flex-col gap-3">
            <p className="text-muted-foreground">
              Click below to return the session to Pytxo Desktop. Your browser may ask to open the
              app.
            </p>
            {deckReturnHref ? (
              <Button asChild>
                <a href={deckReturnHref}>Return to Pytxo Desktop</a>
              </Button>
            ) : (
              <Button disabled>{deckTokenError ? "Token unavailable" : "Preparing return link…"}</Button>
            )}
            {deckTokenError ? (
              <p className="text-sm text-muted-foreground">{deckTokenError}</p>
            ) : null}
          </div>
        )}
        {loading && (
          <p className="text-sm text-muted-foreground">Loading entitlements…</p>
        )}
        {entitlements && !loading && (
          <div className="rounded-xl border border-white/10 bg-background/50 px-4 py-4 text-sm">
            {entitlements.error ? (
              <p className="text-muted-foreground">
                Could not reach Pytxo Link ({entitlements.error}). Local Core tier still works.
              </p>
            ) : (
              <dl className="grid grid-cols-2 gap-x-4 gap-y-3">
                <div>
                  <dt className="text-xs text-muted-foreground">Tier</dt>
                  <dd className="mt-0.5 font-medium capitalize">{entitlements.tier ?? "core"}</dd>
                </div>
                <div>
                  <dt className="text-xs text-muted-foreground">Max agents</dt>
                  <dd className="mt-0.5 font-medium">{entitlements.max_agents ?? 3}</dd>
                </div>
                <div>
                  <dt className="text-xs text-muted-foreground">Cloud</dt>
                  <dd className="mt-0.5 font-medium">
                    {entitlements.cloud_enabled ? "enabled" : "off"}
                  </dd>
                </div>
                {entitlements.org_id ? (
                  <div className="col-span-2">
                    <dt className="text-xs text-muted-foreground">Organization</dt>
                    <dd className="mt-0.5 font-mono text-xs break-all">{entitlements.org_id}</dd>
                  </div>
                ) : null}
                {seats ? (
                  <div className="col-span-2">
                    <dt className="text-xs text-muted-foreground">Seats</dt>
                    <dd className="mt-0.5">
                      {seats.seats_used ?? 0} / {seats.seats_total ?? "-"} used
                      {typeof seats.seats_available === "number" ? (
                        <span className="text-muted-foreground">
                          {" "}
                          ({seats.seats_available} available)
                        </span>
                      ) : null}
                    </dd>
                  </div>
                ) : null}
                {policy?.default_permission_profile ? (
                  <div>
                    <dt className="text-xs text-muted-foreground">Org policy</dt>
                    <dd className="mt-0.5 capitalize">{policy.default_permission_profile}</dd>
                  </div>
                ) : null}
                {policy?.shared_trusted_domains && policy.shared_trusted_domains.length > 0 ? (
                  <div className="col-span-2">
                    <dt className="text-xs text-muted-foreground">Trusted domains</dt>
                    <dd className="mt-0.5 text-xs">{policy.shared_trusted_domains.join(", ")}</dd>
                  </div>
                ) : null}
              </dl>
            )}
          </div>
        )}
        <Link
          href="https://pytxo.com/account#subscription"
          className="text-sm text-primary hover:underline"
        >
          Manage subscription in Paddle portal
        </Link>
      </SignedIn>
    </div>
  );
}
