import Link from "next/link";
import bench from "../../../public/evidence/pybench-1.3.0-2026-10-10.json";

const land = bench.land.summary;
const seconds = (ms: number) => `${(ms / 1000).toFixed(1)} s`;

const BENCH_STATS = [
  [`${land.pytxo_prevented}/${land.naive_landed_broken}`, "broken mains refused: per-branch CI would have landed them"],
  [`${land.correct}/${land.scenarios}`, "scenarios decided as expected"],
  [String(land.false_refusals), "good changes refused"],
  [seconds(land.median_gate_ms), "median gate time, checks included"],
] as const;

export function ProofSection() {
  return <section className="border-b border-white/10" aria-labelledby="proof-title" data-testid="proof-section">
    <div className="mx-auto max-w-[92rem] px-4 py-[clamp(4.5rem,7vw,7.5rem)] sm:px-6 lg:px-10">
      <div className="grid gap-10 lg:grid-cols-[0.8fr_1.4fr]">
        <div>
          <h2 id="proof-title" className="text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">Measured, with the limits printed.</h2>
          <p className="mt-6 max-w-[44ch] text-base leading-relaxed text-[#b4b4bd]">Pybench builds throwaway repositories where agent branches each pass their own checks but break together, and compares what per-branch CI would put on main with what Pytxo decides. It shows the mechanism, not how often it happens in real projects. Run it yourself with <code className="font-mono text-[0.92em]">pytxo bench</code>.</p>
          <p className="mt-4 max-w-[44ch] text-sm leading-relaxed text-[#8d8d96]">Nobody has measured how often agents&apos; branches fail only when combined. The one large study found concurrent pull requests from different agents in about 4.3% of 2,807 repositories. Pytxo catches what your tests catch. Checks passing is evidence, not proof.</p>
          <div className="mt-6 flex flex-wrap gap-x-6 gap-y-6 text-sm">
            <Link href="/bench" className="aperture-link">Pybench results →</Link>
            <Link href="/docs/evidence" className="aperture-link">All evidence and limits →</Link>
          </div>
        </div>
        <div className="grid gap-8 self-start">
          <div className="relative rounded-md border border-white/15 p-2 pt-4" data-testid="proof-bench">
            <span className="absolute -top-2.5 left-4 bg-background px-2 font-mono text-xs text-[#7d7d87]"><b className="font-semibold uppercase tracking-[0.12em] text-[#f5f5f7]">pybench · land</b> · pytxo {bench.pytxo_version} · {bench.machine.os}</span>
            <dl className="grid grid-cols-2 gap-px overflow-hidden rounded-[4px] bg-white/10">
              {BENCH_STATS.map(([value, label]) => <div key={label} className="flex flex-col-reverse bg-[#09090b] p-5 sm:p-6">
                <dt className="mt-2 font-mono text-xs leading-snug text-[#aaaab3]">{label}</dt>
                <dd className="m-0 text-3xl font-semibold tabular-nums tracking-[-0.03em] text-[#f5f5f7] sm:text-4xl">{value}</dd>
              </div>)}
            </dl>
          </div>
        </div>
      </div>
    </div>
  </section>;
}
