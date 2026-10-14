"use client";

import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useState } from "react";
import { SignedIn, SignedOut } from "@clerk/nextjs";

import { Button } from "@/components/ui/button";
import { connectParams, connectPath, returnUrl } from "@/lib/account-connect";
import { SITE_AUTH_ENABLED } from "@/lib/auth-config";


export function AccountConnect() {
  const params = connectParams(useSearchParams());
  const [state, setState] = useState<"idle" | "working" | "sent" | "error">("idle");
  const [message, setMessage] = useState<string | null>(null);

  if (!SITE_AUTH_ENABLED) {
    return <p className="text-sm text-muted-foreground">Sign-in is not available on this site. Pytxo keeps working locally without an account.</p>;
  }
  if (!params) {
    return <div className="flex flex-col gap-3 text-sm">
      <p className="font-medium">This sign-in link is incomplete or expired.</p>
      <p className="text-muted-foreground">Start again from the app: run <code className="font-mono">pytxo login</code>, or use Settings → Account in Pytxo Desktop.</p>
    </div>;
  }
  const who = params.client === "cli" ? "the Pytxo CLI on this computer" : "Pytxo Desktop on this computer";

  async function connect() {
    if (!params || state === "working") return;
    setState("working");
    setMessage(null);
    try {
      const res = await fetch("/api/account/code", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ state: params.state, code_challenge: params.codeChallenge }),
        cache: "no-store",
      });
      if (!res.ok) throw new Error(res.status === 429 ? "Too many attempts. Wait a minute and try again." : "Sign-in is unavailable right now. Try again later.");
      const { code } = (await res.json()) as { code: string };
      setState("sent");
      window.location.assign(returnUrl(params, code));
    } catch (error) {
      setState("error");
      setMessage(error instanceof Error ? error.message : "Sign-in failed.");
    }
  }

  return <div className="flex flex-col gap-5">
    <SignedOut>
      <p className="text-sm text-muted-foreground">Sign in first, then come back here to connect {who}.</p>
      <Button asChild className="self-start">
        <Link href={`/sign-in?return_to=${encodeURIComponent(connectPath(params))}`}>Sign in</Link>
      </Button>
    </SignedOut>
    <SignedIn>
      <p className="text-sm">Connect {who} to your Pytxo account?</p>
      <p className="text-sm text-muted-foreground">Only continue if you just started sign-in from {params.client === "cli" ? <code className="font-mono">pytxo login</code> : "Pytxo Desktop"}. The connection carries no billing or agent access, and you can sign out from the app at any time.</p>
      <Button onClick={() => void connect()} disabled={state === "working" || state === "sent"} className="self-start">
        {state === "working" ? "Connecting…" : state === "sent" ? "Returning to the app…" : "Connect"}
      </Button>
      {state === "sent" && <p role="status" className="text-sm text-muted-foreground">{params.client === "cli" ? "Check your terminal. You can close this tab." : "Your browser may ask to open Pytxo Desktop."}</p>}
      {message && <p role="alert" className="text-sm text-destructive">{message}</p>}
    </SignedIn>
  </div>;
}
