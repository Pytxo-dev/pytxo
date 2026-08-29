/**
 * Transcribed from `tooling/benchmarks/results/signal-real-repo.json`. These
 * values are the only place the site is allowed to state the Signal figure, so
 * a number can never drift between the homepage, the docs, and the source run.
 */
export const SIGNAL_BENCHMARK = {
  /** `summary.weighted_reduction_pct` = 82.9424, reported to one decimal. */
  reductionPct: "82.9",
  files: 185,
  originalBytes: 1_253_675,
  scaffoldedBytes: 213_847,
  medianFileReductionPct: "86.3",
  fallbackFiles: 0,
  measuredOn: "2026-07-30",
  commit: "3acdc77",
  languages: ["Rust", "TypeScript", "JavaScript"],
  corpusRule:
    "Tracked production source under crates, services, packages, Desktop and Web source, and tooling; supported Signal grammars only; tests, fixtures, stories, specs, vendor, dependencies, and generated output excluded.",
  minimumBytes: 1024,
  nonClaims: [
    "Not a model-token saving. Tokenizer output was not measured.",
    "Not a cost saving. No provider bill was compared.",
    "Not a task-success claim. Agent outcome quality was not measured.",
    "Not a general-repository result. One repository, one commit, one host.",
  ],
  caveats: [
    "The measured worktree was dirty at capture time, so the corpus reflects working state rather than a clean checkout of the recorded commit.",
    "Files below 1 KiB and languages without a Signal grammar are excluded, which raises the weighted figure relative to a whole-repository read.",
    "TypeScript reduced least in this corpus at 58.7%; Rust reduced most at 88.0%. A repository weighted toward TypeScript should expect a lower number.",
  ],
  sourcePath: "tooling/benchmarks/results/signal-real-repo.json",
  sourceUrl:
    "https://github.com/Pytxo-dev/pytxo/blob/main/tooling/benchmarks/results/signal-real-repo.json",
} as const;

export const SIGNAL_LANGUAGE_BREAKDOWN = [
  { language: "Rust", files: 125, originalBytes: 1_010_931, scaffoldedBytes: 121_245, reductionPct: "88.0" },
  { language: "TypeScript", files: 54, originalBytes: 221_203, scaffoldedBytes: 91_364, reductionPct: "58.7" },
  { language: "JavaScript", files: 6, originalBytes: 21_541, scaffoldedBytes: 1_238, reductionPct: "94.3" },
] as const;
