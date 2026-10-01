import Link from "next/link";

import { SIGNAL_BENCHMARK } from "@/lib/evidence";

export function MeasuredEvidence() {
  return (
    <section
      className="border-y border-[var(--aperture-line)] bg-[var(--aperture-raised)]"
      aria-labelledby="evidence-title"
      data-testid="evidence-section"
    >
      <div className="section-pad mx-auto max-w-[92rem] lg:px-10">
        <div className="grid gap-10 lg:grid-cols-[minmax(0,18rem)_minmax(0,1fr)] lg:items-end lg:gap-16">
          <div>
            <h2
              id="evidence-title"
              className="max-w-[16ch] text-[clamp(1.75rem,2.8vw,2.5rem)] leading-[1.08] tracking-[-0.04em]"
            >
              One measured number.
            </h2>
            <p
              className="mt-6 text-[clamp(3rem,6vw,5rem)] leading-[0.95] tracking-[-0.045em] text-[#f5f5f7] tabular-nums"
              data-testid="evidence-figure"
            >
              {SIGNAL_BENCHMARK.reductionPct}%
            </p>
          </div>
          <div className="max-w-[40rem]">
            <p className="text-base leading-relaxed text-[#a9a9b2]">
              weighted scaffold-byte reduction across {SIGNAL_BENCHMARK.files}{" "}
              tracked production files. Not tokens, not cost, not quality.
            </p>
            <ul className="mt-6 space-y-2 text-sm leading-relaxed text-[#8d8d96]">
              {SIGNAL_BENCHMARK.nonClaims.slice(0, 3).map((claim) => (
                <li key={claim}>{claim}</li>
              ))}
            </ul>
            <Link
              href="/evidence"
              className="aperture-link mt-6 inline-block text-sm text-[#f5f5f7] transition-colors hover:text-white"
            >
              Read the methodology
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}
