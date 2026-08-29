import { clerkMiddleware } from "@clerk/nextjs/server";
import { NextResponse } from "next/server";

import { SITE_AUTH_BACKEND_ENABLED } from "@/lib/auth-config";

const middleware = SITE_AUTH_BACKEND_ENABLED ? clerkMiddleware() : () => NextResponse.next();

export default middleware;

export const config = {
  matcher: [
    "/((?!_next|docs(?:/|$)|[^?]*\\.(?:html?|css|js(?!on)|jpe?g|webp|png|gif|svg|ttf|woff2?|ico|csv|docx?|xlsx?|zip|webmanifest)).*)",
    "/(api|trpc)(.*)",
    "/__clerk/:path*",
  ],
};
