"use client";

import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";
import { SignedIn, SignedOut, SignUpButton, UserButton } from "@clerk/nextjs";

import { Button } from "@/components/ui/button";
import { SITE_AUTH_ENABLED } from "@/lib/auth-config";
import { desktopAccountUrl, desktopBridgeAllowed, desktopBridgeParams, desktopSignInUrl } from "@/lib/desktop-bridge";

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

function ConfiguredAccountAuth() {
  const [entitlements, setEntitlements] = useState<EntitlementPayload | null>(null);
  const [loading, setLoading] = useState(false);
  const searchParams = useSearchParams();
  const deckCallback = searchParams.get("deck_callback");
  const wantsDesktopReturn = deckCallback === "pytxo-deck";
  const bridge = desktopBridgeParams(searchParams);
  const bridgeEnabled = process.env.NEXT_PUBLIC_ROUTING_BRIDGE_EXPERIMENT === "1";
  const bridgeAllowed = desktopBridgeAllowed(bridge, bridgeEnabled);
  const [returnPending, setReturnPending] = useState(false);
  const [returnError, setReturnError] = useState<string | null>(null);
  const [revokePending, setRevokePending] = useState(false);
  const [revokeMessage, setRevokeMessage] = useState<string | null>(null);

  async function revokeAllRouting() {
    if (revokePending || !window.confirm("Revoke every hosted Routing workspace grant and Desktop routing connection for this account?")) return;
    setRevokePending(true);
    setRevokeMessage(null);
    try {
      const result = await fetch("/api/routing/revoke-all", { method: "DELETE", cache: "no-store" });
      if (!result.ok) throw new Error("unavailable");
      setRevokeMessage("Hosted Routing access revoked. Desktop must reconnect before requesting another grant.");
    } catch {
      setRevokeMessage("Could not confirm revocation. Try again later.");
    } finally {
      setRevokePending(false);
    }
  }

  async function connectDesktop() {
    if (!bridge || returnPending) return;
    setReturnPending(true);
    setReturnError(null);
    try {
      const result = await fetch("/api/routing/desktop-code", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          state: bridge.state,
          code_challenge: bridge.codeChallenge,
          ...(bridge.recoveryOnly ? { recovery_only: true } : {}),
        }),
        cache: "no-store",
      });
      if (!result.ok) throw new Error("Account return is unavailable. Try again later.");
      const payload = (await result.json()) as { code?: unknown };
      if (typeof payload.code !== "string" || !/^pdc1_[A-Za-z0-9_-]{43}$/.test(payload.code)) {
        throw new Error("Account return is unavailable. Try again later.");
      }
      const params = new URLSearchParams({ state: bridge.state, code: payload.code });
      window.location.assign(`pytxo-deck://auth?${params}`);
    } catch {
      setReturnError("Account return is unavailable. Try again later.");
    } finally {
      setReturnPending(false);
    }
  }

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

  const policy = entitlements?.org_policy;
  const seats = entitlements?.org_seats;

  return (
    <div className="flex flex-col gap-4">
      <SignedOut>
        <div className="flex flex-wrap gap-2">
          <Button asChild>
            <Link
              href={desktopSignInUrl(bridge)}
            >
              Sign in
            </Link>
          </Button>
          <SignUpButton
            mode="redirect"
            forceRedirectUrl={desktopAccountUrl(bridge)}
          >
            <Button variant="outline">Sign up</Button>
          </SignUpButton>
        </div>
        <p className="text-sm text-muted-foreground">
          Everything local works without an account. Signing in adds perks: share
          a receipt as a public page, keep shared receipts across machines, and
          publish Pybench results.
        </p>
        <Link href="/plans" className="text-sm text-primary hover:underline">
          Compare plans
        </Link>
      </SignedOut>
      <SignedIn>
        <div className="flex items-center gap-3">
          <UserButton afterSignOutUrl="/" />
          <span className="text-sm text-muted-foreground">
            Signed in. Local Pytxo works the same with or without this.
          </span>
        </div>
        <div className="rounded-xl border border-white/10 bg-background/50 px-4 py-4 text-sm flex flex-col gap-2">
          <p className="font-medium">Connect your tools</p>
          <p className="text-muted-foreground">
            Run <code className="font-mono">pytxo login</code> in a terminal, or open Settings → Account in Pytxo Desktop.
            Then share a receipt with <code className="font-mono">pytxo receipts share &lt;id&gt;</code> or publish
            Pybench with <code className="font-mono">pytxo bench --submit</code>.
          </p>
        </div>
        <SharedReceipts />
        {wantsDesktopReturn && bridge && bridgeAllowed ? (
          <div className="rounded-xl border border-white/10 bg-background/50 px-4 py-4 text-sm flex flex-col gap-3">
            <p className="font-medium">{bridge.recoveryOnly ? "Reconnect Pytxo Desktop to revoke existing access" : "Connect Pytxo Desktop for experimental routing"}</p>
            <p className="text-muted-foreground">
              {bridge.recoveryOnly
                ? "Sign in with the original account for this workspace. Desktop will only accept that account, and new hosted grants remain controlled by its experimental switches."
                : "This connects your Pytxo account to Desktop. Hosted Jev routing stays off until you separately allow a reviewed workspace packet in Desktop."}
            </p>
            <Button onClick={() => void connectDesktop()} disabled={returnPending}>
              {returnPending ? "Connecting…" : "Return to Desktop"}
            </Button>
            {returnError && <p role="alert" className="text-destructive">{returnError}</p>}
          </div>
        ) : wantsDesktopReturn && (
          <div className="rounded-xl border border-white/10 bg-background/50 px-4 py-4 text-sm flex flex-col gap-3">
            <p className="font-medium">Desktop account return is unavailable here.</p>
            <p className="text-muted-foreground">
              Start sign-in from Desktop to create a fresh one-time request. Local Core features remain available without an account.
            </p>
          </div>
        )}
        {bridgeEnabled && <div className="rounded-xl border border-white/10 bg-background/50 px-4 py-4 text-sm flex flex-col gap-3">
            <p className="font-medium">Hosted Routing access</p>
            <p className="text-muted-foreground">If you used experimental Routing and a Desktop is lost, revoke all Routing credentials, workspace grants, and evaluation tokens here. This affects every connected Desktop.</p>
            <Button variant="outline" onClick={() => void revokeAllRouting()} disabled={revokePending}>
              {revokePending ? "Revoking…" : "Revoke all Routing access"}
            </Button>
            {revokeMessage && <p role="status">{revokeMessage}</p>}
          </div>}
        {loading && (
          <p className="text-sm text-muted-foreground">Loading entitlements…</p>
        )}
        {entitlements && !loading && (entitlements.error || entitlements.org_id) && (
          <div className="rounded-xl border border-white/10 bg-background/50 px-4 py-4 text-sm">
            {entitlements.error ? (
              <p className="text-muted-foreground">
                Could not load organization details ({entitlements.error}). Local Pytxo still works.
              </p>
            ) : (
              <dl className="grid grid-cols-2 gap-x-4 gap-y-3">
                <div>
                  <dt className="text-xs text-muted-foreground">Plan</dt>
                  <dd className="mt-0.5 font-medium capitalize">{entitlements.tier ?? "free"}</dd>
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
        <p className="text-sm text-muted-foreground">
          Billing self-service is not connected. Use the support contact on your
          payment receipt for changes or cancellation.
        </p>
      </SignedIn>
    </div>
  );
}

export function AccountAuth() {
  if (!SITE_AUTH_ENABLED) {
    return (
      <div className="flex flex-col gap-3">
        <p className="text-sm text-muted-foreground">
          Account sync is not configured on this deployment. Pytxo Core,
          Desktop, and local provider sessions work without an account.
        </p>
        <div className="flex flex-wrap gap-2">
          <Button asChild>
            <Link href="/download">Download Pytxo</Link>
          </Button>
          <Button variant="outline" asChild>
            <Link href="/docs/getting-started/desktop-setup">
              Set up Desktop
            </Link>
          </Button>
        </div>
      </div>
    );
  }

  return <ConfiguredAccountAuth />;
}

type SharedReceiptRow = {
  id: string;
  created_at: string;
  payload: { decision?: string; kind?: string; inputs?: number; files_changed?: number };
};

/** Account perk: receipts this account has shared, newest first. */
function SharedReceipts() {
  const [rows, setRows] = useState<SharedReceiptRow[] | null>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    let cancelled = false;
    fetch("/api/account/receipts", { cache: "no-store" })
      .then((res) => (res.ok ? res.json() : Promise.reject(new Error(String(res.status)))))
      .then((data: SharedReceiptRow[]) => !cancelled && setRows(data))
      .catch(() => !cancelled && setFailed(true));
    return () => {
      cancelled = true;
    };
  }, []);
  return (
    <div className="rounded-xl border border-white/10 bg-background/50 px-4 py-4 text-sm flex flex-col gap-3" data-testid="shared-receipts">
      <p className="font-medium">Shared receipts</p>
      {failed ? (
        <p className="text-muted-foreground">Could not load shared receipts right now.</p>
      ) : rows === null ? (
        <p className="text-muted-foreground">Loading…</p>
      ) : rows.length === 0 ? (
        <p className="text-muted-foreground">None yet. Shared receipts appear here and at their public link.</p>
      ) : (
        <ul className="flex flex-col divide-y divide-white/10">
          {rows.map((row) => (
            <li key={row.id} className="flex flex-wrap items-baseline justify-between gap-2 py-2">
              <Link href={`/r/${row.id}`} className="font-mono text-xs text-foreground hover:underline">{row.id}</Link>
              <span className="text-xs text-muted-foreground">
                {row.payload.kind ?? "land"} · {row.payload.decision ?? "?"} · {row.payload.inputs ?? 0} inputs · {row.payload.files_changed ?? 0} files · {row.created_at.slice(0, 10)}
              </span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
