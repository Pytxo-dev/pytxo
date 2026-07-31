import assert from "node:assert/strict";
import test from "node:test";

import { classifyChange } from "../src/risk-policy.mjs";

test("routine source changes stay local", () => {
  assert.deepEqual(classifyChange({ paths: ["src/view.mjs"] }), {
    level: "routine",
    requiresApproval: false,
  });
});

test("deployment changes require review", () => {
  assert.deepEqual(classifyChange({ paths: ["deploy/release.mjs"] }), {
    level: "review",
    requiresApproval: true,
  });
});
