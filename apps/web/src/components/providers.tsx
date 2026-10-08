"use client";

import { ClerkProvider } from "@clerk/nextjs";
import { shadcn } from "@clerk/themes";
import type { ReactNode } from "react";

import { SITE_AUTH_ENABLED } from "@/lib/auth-config";

export function Providers({ children }: { children: ReactNode }) {
  if (!SITE_AUTH_ENABLED) {
    return children;
  }

  return (
    <ClerkProvider
      appearance={{
        theme: shadcn,
        cssLayerName: "clerk",
        variables: { borderRadius: "6px", fontFamily: "inherit" },
      }}
      localization={{
        signIn: { start: { title: "Sign in to Pytxo", subtitle: "Link Pytxo Desktop and see what your account includes." } },
        signUp: { start: { title: "Create your Pytxo account", subtitle: "Pytxo runs locally without one. An account links Desktop to hosted features." } },
      }}
      signInUrl="/sign-in"
      signUpUrl="/sign-up"
      signInFallbackRedirectUrl="/account"
      signUpFallbackRedirectUrl="/account"
    >
      {children}
    </ClerkProvider>
  );
}
