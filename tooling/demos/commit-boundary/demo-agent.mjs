import { existsSync, writeFileSync } from "node:fs";

if (!existsSync(".pytxo-example-v1")) {
  throw new Error("Refusing to run outside the Pytxo guided-example fixture");
}

writeFileSync(
  "src/risk-policy.mjs",
  `const REVIEW_PATHS = ["deploy/", "billing/", ".github/workflows/"];
const NETWORK_PATHS = ["infra/network/", "network/"];
const DESTRUCTIVE_PATHS = ["scripts/reset", "scripts/destroy"];

export function classifyChange({ paths = [] } = {}) {
  const normalized = paths.map((path) => path.replaceAll("\\\\", "/").toLowerCase());
  const networkChange = normalized.some((path) =>
    NETWORK_PATHS.some((prefix) => path.startsWith(prefix)),
  );
  const destructiveChange = normalized.some((path) =>
    DESTRUCTIVE_PATHS.some((prefix) => path.startsWith(prefix)),
  );
  const requiresReview = networkChange || destructiveChange || normalized.some((path) =>
    REVIEW_PATHS.some((prefix) => path.startsWith(prefix)),
  );

  const summary = destructiveChange
    ? "Destructive command path; confirm recovery before Apply."
    : networkChange
      ? "Network boundary changed; confirm intended egress before Apply."
      : requiresReview
        ? "Sensitive path changed; review the prepared bytes before Apply."
        : "Routine source change; no elevated boundary detected.";

  return {
    level: requiresReview ? "review" : "routine",
    requiresApproval: requiresReview,
    summary,
  };
}
`,
);

writeFileSync(
  "test/risk-policy.test.mjs",
  `import assert from "node:assert/strict";
import test from "node:test";

import { classifyChange } from "../src/risk-policy.mjs";

test("routine source changes stay local", () => {
  assert.deepEqual(classifyChange({ paths: ["src/view.mjs"] }), {
    level: "routine",
    requiresApproval: false,
    summary: "Routine source change; no elevated boundary detected.",
  });
});

test("network changes name the egress risk", () => {
  assert.deepEqual(classifyChange({ paths: ["infra/network/firewall.mjs"] }), {
    level: "review",
    requiresApproval: true,
    summary: "Network boundary changed; confirm intended egress before Apply.",
  });
});

test("destructive command changes name recovery", () => {
  assert.deepEqual(classifyChange({ paths: ["scripts/reset-database.mjs"] }), {
    level: "review",
    requiresApproval: true,
    summary: "Destructive command path; confirm recovery before Apply.",
  });
});
`,
);

writeFileSync(
  "README.md",
  `# Approval risk demo

This dependency-free fixture demonstrates Pytxo's reviewed repository boundary.

## Examples

- A change under \`infra/network/\` requires review and names the egress risk.
- A change under \`scripts/reset*\` requires review and asks for a recovery check.

Run \`npm test\` to verify the independently observable post-state.
`,
);

console.log("deterministic demo adapter prepared risk policy, tests, and documentation");
