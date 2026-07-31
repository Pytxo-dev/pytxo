"use client";

import { ClerkProvider } from "@clerk/nextjs";
import { shadcn } from "@clerk/ui/themes";
import type { ReactNode } from "react";

import { SITE_AUTH_ENABLED } from "@/lib/auth-config";

export function Providers({ children }: { children: ReactNode }) {
  if (!SITE_AUTH_ENABLED) {
    return children;
  }

  return (
    <ClerkProvider
      appearance={{ theme: shadcn, cssLayerName: "clerk" }}
      signInUrl="/sign-in"
      signUpUrl="/sign-up"
      signInFallbackRedirectUrl="/account"
      signUpFallbackRedirectUrl="/account"
    >
      {children}
    </ClerkProvider>
  );
}
