import Link from "next/link";

import { SIGNAL_BENCHMARK } from "@/lib/evidence";

export function MeasuredEvidence() {
  return (
    <section
      className="border-y border-[var(--aperture-line)] bg-[var(--aperture-raised)]"
      aria-labelledby="evidence-title"
      data-testid="evidence-section"
    >
      <div className="section-pad mx-auto grid max-w-[92rem] gap-14 lg:grid-cols-[1.1fr_0.9fr] lg:gap-20 lg:px-10">
        <div>
          <h2
            id="evidence-title"
            className="max-w-[24ch] text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]"
          >
            One measured number, with its limits attached.
          </h2>
          {/*
            The display face, not mono. Mono is reserved for machine values, and
            a monospaced decimal point at 6rem opens a cell-wide gap that reads
            as "82 . 9%".
          */}
          <p
            className="mt-10 text-[clamp(3.5rem,7vw,6rem)] leading-[0.95] tracking-[-0.045em] text-[#f5f5f7] tabular-nums"
            data-testid="evidence-figure"
          >
            {SIGNAL_BENCHMARK.reductionPct}%
          </p>
          <p className="mt-6 max-w-[38rem] text-base leading-relaxed text-[#a9a9b2]">
            weighted scaffold-byte reduction across {SIGNAL_BENCHMARK.files}{" "}
            tracked production files. Signal Core sends syntax structure first, so an
            agent&apos;s first read of a file is its shape rather than its full source.
          </p>
          <dl className="mt-10 grid max-w-[38rem] grid-cols-2 gap-x-8 gap-y-6 border-t border-[var(--aperture-line)] pt-8">
            <div>
              <dt className="font-mono text-[12px] uppercase tracking-[0.05em] text-[#6f6f79]">
                Input
              </dt>
              <dd className="mt-2 font-mono text-[15px] tabular-nums text-[#c7c7ce]">
                {SIGNAL_BENCHMARK.originalBytes.toLocaleString("en-US")} bytes
              </dd>
            </div>
            <div>
              <dt className="font-mono text-[12px] uppercase tracking-[0.05em] text-[#6f6f79]">
                Scaffolded
              </dt>
              <dd className="mt-2 font-mono text-[15px] tabular-nums text-[#c7c7ce]">
                {SIGNAL_BENCHMARK.scaffoldedBytes.toLocaleString("en-US")} bytes
              </dd>
            </div>
            <div>
              <dt className="font-mono text-[12px] uppercase tracking-[0.05em] text-[#6f6f79]">
                Measured
              </dt>
              <dd className="mt-2 font-mono text-[15px] text-[#c7c7ce]">
                {SIGNAL_BENCHMARK.measuredOn}
              </dd>
            </div>
            <div>
              <dt className="font-mono text-[12px] uppercase tracking-[0.05em] text-[#6f6f79]">
                Fallbacks
              </dt>
              <dd className="mt-2 font-mono text-[15px] tabular-nums text-[#c7c7ce]">
                {SIGNAL_BENCHMARK.fallbackFiles}
              </dd>
            </div>
          </dl>
        </div>

        <div className="aperture-panel h-fit p-7 sm:p-9">
          <h3 className="text-lg tracking-[-0.02em]">What this number is not</h3>
          <ul className="mt-6 space-y-4 text-sm leading-relaxed text-[#8d8d96]">
            {SIGNAL_BENCHMARK.nonClaims.map((claim) => (
              <li key={claim} className="flex gap-3">
                <span aria-hidden className="mt-[0.55em] size-[9px] shrink-0 rounded-[2px] border border-[var(--state-unknown)] bg-[repeating-linear-gradient(45deg,var(--state-unknown)_0_1px,transparent_1px_3px)]" />
                <span>{claim}</span>
              </li>
            ))}
          </ul>
          <div className="mt-8 flex flex-col gap-3 border-t border-[var(--aperture-line)] pt-7">
            <Link
              href="/evidence"
              className="aperture-link self-start text-sm text-[#f5f5f7] transition-colors hover:text-white"
            >
              Read the methodology
            </Link>
            <Link
              href={SIGNAL_BENCHMARK.sourceUrl}
              className="aperture-link self-start font-mono text-[12px] text-[#7d7d87] transition-colors hover:text-white"
            >
              {SIGNAL_BENCHMARK.sourcePath}
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}
