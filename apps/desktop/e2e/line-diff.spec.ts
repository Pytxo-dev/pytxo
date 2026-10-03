import { expect, test } from "@playwright/test";
import { checkLineDiff, diffLines, diffStats, foldUnchanged } from "../src/lib/line-diff";

test("review diffs mark added, removed and folded unchanged lines", () => {
  checkLineDiff();
});

test("Review preserves newline-only changes and can reconstruct both exact inputs", () => {
  const cases = [
    ["value=1", "value=1\n"],
    ["value=1\n", "value=1"],
    ["a\r\nb\r\n", "a\nb\n"],
    ["a\r\nb\n", "a\nb\r\n"],
    ["", "\n"],
    ["x\n", "x\n\n"],
    ["a\rb", "a\rb\n"],
  ];
  for (const [before, after] of cases) {
    const lines = diffLines(before, after)!;
    const reconstruct = (side: "before" | "after") => lines
      .filter(line => line.kind !== (side === "before" ? "add" : "del"))
      .map(line => line.text + ({ lf: "\n", crlf: "\r\n", none: "" }[line.ending]))
      .join("");
    expect(reconstruct("before")).toBe(before);
    expect(reconstruct("after")).toBe(after);
    expect(diffStats(lines).added + diffStats(lines).removed).toBeGreaterThan(0);
    expect(foldUnchanged(lines).some(line => line.kind === "add" || line.kind === "del")).toBe(true);
  }
  expect(diffLines("value=1", "value=1\n")).toEqual([
    { kind: "del", text: "value=1", before: 1, ending: "none" },
    { kind: "add", text: "value=1", after: 1, ending: "lf" },
  ]);
});
