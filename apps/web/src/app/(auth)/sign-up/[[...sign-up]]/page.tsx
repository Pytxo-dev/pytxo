"use client";

import { SignUp } from "@clerk/nextjs";
import { useSearchParams } from "next/navigation";

function accountRedirect(deckCallback: string | null): string {
  if (deckCallback === "pytxo-deck") {
    return "/account?deck_callback=pytxo-deck";
  }
  return "/account";
}

export default function SignUpPage() {
  const searchParams = useSearchParams();
  const deckCallback = searchParams.get("deck_callback");
  const redirectUrl = accountRedirect(deckCallback);

  return (
    <SignUp
      routing="path"
      path="/sign-up"
      signInUrl={
        deckCallback === "pytxo-deck"
          ? "/sign-in?deck_callback=pytxo-deck"
          : "/sign-in"
      }
      forceRedirectUrl={redirectUrl}
      fallbackRedirectUrl={redirectUrl}
    />
  );
}
