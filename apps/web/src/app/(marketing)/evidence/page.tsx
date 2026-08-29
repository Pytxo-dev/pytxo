import type { Metadata } from "next";
import Link from "next/link";

import { StateChip } from "@/components/site/state-chip";
import { SIGNAL_BENCHMARK, SIGNAL_LANGUAGE_BREAKDOWN } from "@/lib/evidence";

export const metadata: Metadata = {
  title: "Evidence",
  description:
    "How Pytxo's published Signal Core figure was measured, what it covers, and what it does not claim.",
};

export default function EvidencePage() {
  return (
    <div className="mx-auto max-w-[76rem] px-4 py-16 sm:px-6 sm:py-24 lg:px-10">
      <h1 className="max-w-[24ch] text-[clamp(2.25rem,4vw,3.75rem)] leading-[1.04] tracking-[-0.045em]">
        Evidence and its limits.
      </h1>
      <p className="mt-7 max-w-[46rem] text-base leading-relaxed text-[#a9a9b2] sm:text-lg">
        Pytxo publishes one performance figure. This page states how it was produced, what
        the corpus was, and which conclusions it does not support, so you can decide
        whether it transfers to your repository.
      </p>

      <section className="mt-16 border-t border-[var(--aperture-line)] pt-10" aria-labelledby="figure-title">
        <h2 id="figure-title" className="text-2xl tracking-[-0.025em]">
          The figure
        </h2>
        <div className="mt-8 grid gap-x-10 gap-y-8 sm:grid-cols-2 lg:grid-cols-4">
          {[
            { label: "Weighted reduction", value: `${SIGNAL_BENCHMARK.reductionPct}%` },
            { label: "Median per file", value: `${SIGNAL_BENCHMARK.medianFileReductionPct}%` },
            { label: "Files in corpus", value: String(SIGNAL_BENCHMARK.files) },
            { label: "Raw fallbacks", value: String(SIGNAL_BENCHMARK.fallbackFiles) },
          ].map((stat) => (
            <div key={stat.label}>
              <dt className="font-mono text-[12px] uppercase tracking-[0.05em] text-[#6f6f79]">
                {stat.label}
              </dt>
              {/* Display face: a monospaced decimal point at this size reads as a gap. */}
              <dd className="mt-3 text-[clamp(1.75rem,3vw,2.5rem)] tabular-nums leading-none tracking-[-0.03em] text-[#f5f5f7]">
                {stat.value}
              </dd>
            </div>
          ))}
        </div>
        <p className="mt-10 max-w-[52rem] text-sm leading-relaxed text-[#8d8d96]">
          {SIGNAL_BENCHMARK.originalBytes.toLocaleString("en-US")} bytes of source were
          reduced to {SIGNAL_BENCHMARK.scaffoldedBytes.toLocaleString("en-US")} bytes of
          structural scaffold. Measured {SIGNAL_BENCHMARK.measuredOn} at commit{" "}
          <code className="font-mono text-[#c7c7ce]">{SIGNAL_BENCHMARK.commit}</code> on a
          single Windows host.
        </p>
      </section>

      <section className="mt-16 border-t border-[var(--aperture-line)] pt-10" aria-labelledby="corpus-title">
        <h2 id="corpus-title" className="text-2xl tracking-[-0.025em]">
          The corpus
        </h2>
        <p className="mt-6 max-w-[52rem] text-sm leading-relaxed text-[#a9a9b2]">
          {SIGNAL_BENCHMARK.corpusRule} Files under{" "}
          {SIGNAL_BENCHMARK.minimumBytes.toLocaleString("en-US")} bytes were excluded.
        </p>
        <div className="mt-10 overflow-hidden rounded-[8px] border border-[var(--aperture-line)]">
          <div className="hidden grid-cols-[minmax(0,1fr)_5rem_minmax(0,0.9fr)_minmax(0,0.9fr)_6rem] gap-6 border-b border-[var(--aperture-line)] bg-[var(--aperture-raised)] px-6 py-3 font-mono text-[11px] uppercase tracking-[0.05em] text-[#6f6f79] sm:grid">
            <span>Language</span>
            <span className="text-right">Files</span>
            <span className="text-right">Source bytes</span>
            <span className="text-right">Scaffold bytes</span>
            <span className="text-right">Reduction</span>
          </div>
          {SIGNAL_LANGUAGE_BREAKDOWN.map((row) => (
            <div
              key={row.language}
              className="grid gap-1 border-b border-[var(--aperture-line)] px-6 py-4 last:border-b-0 sm:grid-cols-[minmax(0,1fr)_5rem_minmax(0,0.9fr)_minmax(0,0.9fr)_6rem] sm:items-center sm:gap-6"
            >
              <span className="text-[15px] text-[#f5f5f7]">{row.language}</span>
              <span className="font-mono text-[13px] tabular-nums text-[#a9a9b2] sm:text-right">
                {row.files}
              </span>
              <span className="font-mono text-[13px] tabular-nums text-[#7d7d87] sm:text-right">
                {row.originalBytes.toLocaleString("en-US")}
              </span>
              <span className="font-mono text-[13px] tabular-nums text-[#7d7d87] sm:text-right">
                {row.scaffoldedBytes.toLocaleString("en-US")}
              </span>
              <span className="font-mono text-[13px] tabular-nums text-[#c7c7ce] sm:text-right">
                {row.reductionPct}%
              </span>
            </div>
          ))}
        </div>
      </section>

      <section className="mt-16 border-t border-[var(--aperture-line)] pt-10" aria-labelledby="nonclaims-title">
        <h2 id="nonclaims-title" className="text-2xl tracking-[-0.025em]">
          What it does not claim
        </h2>
        <ul className="mt-8 grid gap-x-10 gap-y-6 sm:grid-cols-2">
          {SIGNAL_BENCHMARK.nonClaims.map((claim) => (
            <li key={claim} className="flex gap-3">
              <StateChip tone="unknown" label="Not measured" className="shrink-0" />
              <span className="text-sm leading-relaxed text-[#a9a9b2]">{claim}</span>
            </li>
          ))}
        </ul>
      </section>

      <section className="mt-16 border-t border-[var(--aperture-line)] pt-10" aria-labelledby="caveats-title">
        <h2 id="caveats-title" className="text-2xl tracking-[-0.025em]">
          Caveats that weaken the number
        </h2>
        <ul className="mt-8 max-w-[52rem] space-y-5">
          {SIGNAL_BENCHMARK.caveats.map((caveat) => (
            <li
              key={caveat}
              className="border-l-2 border-[var(--state-claimed)] pl-5 text-sm leading-relaxed text-[#a9a9b2]"
            >
              {caveat}
            </li>
          ))}
        </ul>
      </section>

      <section className="mt-16 border-t border-[var(--aperture-line)] pt-10" aria-labelledby="reproduce-title">
        <h2 id="reproduce-title" className="text-2xl tracking-[-0.025em]">
          Reproduce it
        </h2>
        <p className="mt-6 max-w-[52rem] text-sm leading-relaxed text-[#a9a9b2]">
          The raw result, including the per-file table for every one of the{" "}
          {SIGNAL_BENCHMARK.files} files, is committed in the repository.
        </p>
        <div className="mt-8 flex flex-col gap-4">
          <Link
            href={SIGNAL_BENCHMARK.sourceUrl}
            className="aperture-link self-start font-mono text-[13px] text-[#f5f5f7] transition-colors hover:text-white"
          >
            {SIGNAL_BENCHMARK.sourcePath}
          </Link>
          <Link
            href="/docs/concepts/signal-core"
            className="aperture-link self-start text-sm text-[#a9a9b2] transition-colors hover:text-white"
          >
            How Signal Core builds a scaffold
          </Link>
        </div>
      </section>
    </div>
  );
}
