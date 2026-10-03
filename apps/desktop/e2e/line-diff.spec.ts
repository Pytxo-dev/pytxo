import { test } from "@playwright/test";
import { checkLineDiff } from "../src/lib/line-diff";

test("review diffs mark added, removed and folded unchanged lines", () => {
  checkLineDiff();
});
