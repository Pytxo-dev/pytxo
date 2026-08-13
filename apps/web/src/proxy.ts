import { clerkMiddleware } from "@clerk/nextjs/server";
import { NextResponse } from "next/server";

const middleware =
  process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY && process.env.CLERK_SECRET_KEY
  ? clerkMiddleware()
  : () => NextResponse.next();

export default middleware;

export const config = {
  matcher: [
    "/((?!_next|docs(?:/|$)|[^?]*\\.(?:html?|css|js(?!on)|jpe?g|webp|png|gif|svg|ttf|woff2?|ico|csv|docx?|xlsx?|zip|webmanifest)).*)",
    "/(api|trpc)(.*)",
    "/__clerk/:path*",
  ],
};
