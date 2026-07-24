"use client";

import { SignIn } from "@clerk/nextjs";
import { useSearchParams } from "next/navigation";

function accountRedirect(deckCallback: string | null): string {
  if (deckCallback === "pytxo-deck") {
    return "/account?deck_callback=pytxo-deck";
  }
  return "/account";
}

export default function SignInPage() {
  const searchParams = useSearchParams();
  const deckCallback = searchParams.get("deck_callback");
  const redirectUrl = accountRedirect(deckCallback);

  return (
    <SignIn
      routing="path"
      path="/sign-in"
      signUpUrl={
        deckCallback === "pytxo-deck"
          ? "/sign-up?deck_callback=pytxo-deck"
          : "/sign-up"
      }
      forceRedirectUrl={redirectUrl}
      fallbackRedirectUrl={redirectUrl}
    />
  );
}
