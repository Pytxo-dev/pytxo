/**
 * Shared by server and client components, so it may only read variables that
 * exist in both bundles. `CLERK_SECRET_KEY` is server-only: including it here
 * made the flag true during SSR and false after hydration, which silently
 * dropped the whole auth cluster out of the header and produced a hydration
 * mismatch on every page.
 */
export const SITE_AUTH_ENABLED = Boolean(process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY);

/**
 * Server-only gate for code paths that actually call the Clerk backend. Never
 * import this from a client component.
 */
export const SITE_AUTH_BACKEND_ENABLED =
  SITE_AUTH_ENABLED && Boolean(process.env.CLERK_SECRET_KEY);
