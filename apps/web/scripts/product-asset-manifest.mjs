/**
 * Single source of truth for which Desktop captures exist, which of them the
 * marketing site is allowed to serve, and at which sizes. `sync-product-assets`
 * copies from it and `verify-product-assets` checks against it, so a route
 * rename cannot leave the two out of step.
 */
export const REFERENCE_ROUTES = [
  "work",
  "history",
  "flow",
  "approvals",
  "setup",
  "integrations",
  "workspaces",
  "run-review",
  "run-applied",
];

export const VIEWPORTS = [
  { slug: "1600x1000", width: 1600, height: 1000 },
  { slug: "1280x800", width: 1280, height: 800 },
  { slug: "960x640", width: 960, height: 640 },
];

/**
 * The homepage now leads with one large, legible Work capture instead of a wall
 * of thumbnails, so only the routes actually published are mirrored.
 */
export const MARKETING_ROUTES = ["work", "history", "approvals", "run-review", "run-applied"];

export const MARKETING_VIEWPORTS = VIEWPORTS.filter(({ slug }) => slug !== "1280x800");
