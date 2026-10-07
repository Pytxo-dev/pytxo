/** One line of a unified diff; numbers are 1-based and absent on the side the line is not in. */
type LineContent = { text: string; ending: "lf" | "crlf" | "none" };
export type DiffLine = LineContent & (
  | { kind: "same"; text: string; before: number; after: number }
  | { kind: "add"; text: string; after: number }
  | { kind: "del"; text: string; before: number });

/** A run of unchanged lines folded away from the visible context. */
export type DiffFold = { kind: "fold"; count: number; lines: DiffLine[] };

export const MAX_DIFF_EDITS = 1000;

function splitLines(text: string) {
  if (text === "") return [];
  const lines = text.split("\n").map((line, index, all) => index < all.length - 1 ? `${line}\n` : line);
  if (lines.at(-1) === "") lines.pop();
  return lines;
}

function lineContent(raw: string): LineContent {
  if (raw.endsWith("\r\n")) return { text: raw.slice(0, -2), ending: "crlf" };
  if (raw.endsWith("\n")) return { text: raw.slice(0, -1), ending: "lf" };
  return { text: raw, ending: "none" };
}

/**
 * Line diff of two texts (Myers, after trimming the common prefix and suffix).
 * Returns null when the files differ in more than `maxEdits` lines, so a
 * caller can show the exact before/after instead of a slow or huge diff.
 */
// ponytail: trace memory grows with the edit cap; switch to linear-space Myers if large rewrites must diff.
export function diffLines(beforeText: string, afterText: string, maxEdits = MAX_DIFF_EDITS): DiffLine[] | null {
  const a = splitLines(beforeText);
  const b = splitLines(afterText);
  let start = 0;
  while (start < a.length && start < b.length && a[start] === b[start]) start++;
  let endA = a.length;
  let endB = b.length;
  while (endA > start && endB > start && a[endA - 1] === b[endB - 1]) { endA--; endB--; }

  const n = endA - start;
  const m = endB - start;
  const max = n + m;
  // The edit count bounds both time and the trace's memory (limit² Int32s).
  const limit = Math.min(max, maxEdits);
  if (limit < Math.abs(n - m)) return null;
  const offset = limit + 1;
  const v = new Int32Array(2 * limit + 3);
  const trace: Int32Array[] = [];
  let found = max === 0;
  for (let d = 0; d <= max && !found; d++) {
    if (d > maxEdits) return null;
    trace.push(v.slice());
    for (let k = -d; k <= d; k += 2) {
      let x = k === -d || (k !== d && v[offset + k - 1] < v[offset + k + 1]) ? v[offset + k + 1] : v[offset + k - 1] + 1;
      let y = x - k;
      while (x < n && y < m && a[start + x] === b[start + y]) { x++; y++; }
      v[offset + k] = x;
      if (x >= n && y >= m) { found = true; break; }
    }
  }

  // Walk the trace backwards from (n, m) to recover the edit script.
  const middle: DiffLine[] = [];
  let x = n;
  let y = m;
  for (let d = trace.length - 1; d >= 0 && (x > 0 || y > 0); d--) {
    const previous = trace[d];
    const k = x - y;
    const down = k === -d || (k !== d && previous[offset + k - 1] < previous[offset + k + 1]);
    const prevK = down ? k + 1 : k - 1;
    const prevX = d === 0 ? 0 : previous[offset + prevK];
    const prevY = prevX - prevK;
    while (x > prevX && y > prevY) { x--; y--; middle.push({ kind: "same", ...lineContent(a[start + x]), before: start + x + 1, after: start + y + 1 }); }
    if (d === 0) break;
    if (down) { y--; middle.push({ kind: "add", ...lineContent(b[start + y]), after: start + y + 1 }); }
    else { x--; middle.push({ kind: "del", ...lineContent(a[start + x]), before: start + x + 1 }); }
  }
  middle.reverse();

  const lines: DiffLine[] = [];
  for (let i = 0; i < start; i++) lines.push({ kind: "same", ...lineContent(a[i]), before: i + 1, after: i + 1 });
  lines.push(...middle);
  for (let i = 0; i < a.length - endA; i++) lines.push({ kind: "same", ...lineContent(a[endA + i]), before: endA + i + 1, after: endB + i + 1 });
  return lines;
}

/** Keeps `context` unchanged lines around each change and folds the rest. */
export function foldUnchanged(lines: DiffLine[], context = 3): Array<DiffLine | DiffFold> {
  const keep = lines.map(() => false);
  lines.forEach((line, index) => {
    if (line.kind === "same") return;
    for (let i = Math.max(0, index - context); i <= Math.min(lines.length - 1, index + context); i++) keep[i] = true;
  });
  const out: Array<DiffLine | DiffFold> = [];
  let folded: DiffLine[] = [];
  const flush = () => {
    // Folding fewer than two lines saves no space.
    if (folded.length > 1) out.push({ kind: "fold", count: folded.length, lines: folded });
    else out.push(...folded);
    folded = [];
  };
  lines.forEach((line, index) => {
    if (keep[index]) { flush(); out.push(line); } else folded.push(line);
  });
  flush();
  return out;
}

export function diffStats(lines: DiffLine[]) {
  return {
    added: lines.filter((line) => line.kind === "add").length,
    removed: lines.filter((line) => line.kind === "del").length,
  };
}

/** Runnable check used by e2e/line-diff.spec.ts. */
export function checkLineDiff() {
  const assert = (ok: boolean, message: string) => { if (!ok) throw new Error(message); };
  const render = (lines: DiffLine[] | null) => (lines ?? []).map((line) => `${line.kind === "add" ? "+" : line.kind === "del" ? "-" : " "}${line.text}`).join("\n");

  assert(render(diffLines("a\nb\nc\n", "a\nB\nc\n")) === " a\n-b\n+B\n c", "single-line change");
  assert(render(diffLines("", "x\ny")) === "+x\n+y", "added file");
  assert(render(diffLines("x\ny\n", "")) === "-x\n-y", "deleted file");
  assert(render(diffLines("same\r\n", "same\n")) === "-same\n+same", "line endings are exact changes");
  const moved = diffLines("1\n2\n3\n4\n5\n", "0\n1\n2\n4\n5\n6\n");
  assert(render(moved) === "+0\n 1\n 2\n-3\n 4\n 5\n+6", "insert, delete and append");
  const numbered = moved!.find((line) => line.kind === "same" && line.text === "4");
  assert(!!numbered && numbered.kind === "same" && numbered.before === 4 && numbered.after === 4, "line numbers follow both sides");
  assert(diffLines("a\nb\nc", "x\ny\nz", 2) === null, "edit cap refuses large rewrites");
  const stats = diffStats(moved!);
  assert(stats.added === 2 && stats.removed === 1, "stats count added and removed lines");

  const long = Array.from({ length: 20 }, (_, i) => `line ${i}`);
  const changed = [...long];
  changed[10] = "changed";
  const folded = foldUnchanged(diffLines(long.join("\n"), changed.join("\n"))!);
  assert(folded[0].kind === "fold" && folded[0].count === 7, "leading unchanged lines fold");
  assert(folded.at(-1)!.kind === "fold" && (folded.at(-1) as DiffFold).count === 6, "trailing unchanged lines fold");
  assert(folded.filter((item) => item.kind !== "fold").length === 8, "three context lines kept on each side");
}
