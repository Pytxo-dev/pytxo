const REVIEW_PATHS = ["deploy/", "billing/", ".github/workflows/"];

export function classifyChange({ paths = [] } = {}) {
  const normalized = paths.map((path) => path.replaceAll("\\", "/").toLowerCase());
  const requiresReview = normalized.some((path) =>
    REVIEW_PATHS.some((prefix) => path.startsWith(prefix)),
  );

  return {
    level: requiresReview ? "review" : "routine",
    requiresApproval: requiresReview,
  };
}
