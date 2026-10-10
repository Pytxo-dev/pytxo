import Link from "next/link";
import live from "../../../public/evidence/fleet-live-2026-10-10.json";
import bench from "../../../public/evidence/pybench-1.3.0-2026-10-10.json";

const minutes = (seconds: number) => `${Math.floor(seconds / 60)} min ${seconds % 60} s`;
const land = bench.land.summary;
const seconds = (ms: number) => `${(ms / 1000).toFixed(1)} s`;

const BENCH_STATS = [
  [`${land.pytxo_prevented}/${land.naive_landed_broken}`, "broken mains per-branch CI would have landed, refused"],
  [`${land.correct}/${land.scenarios}`, "scenarios decided as expected"],
  [String(land.false_refusals), "good changes refused"],
  [seconds(land.median_gate_ms), "median gate time, checks included"],
] as const;

const LIVE_STATS = [
  [String(live.plan.tasks), `tasks from one paragraph, split by ${live.split_by}`],
  [String(live.agent_clis.length), "agent CLIs, live, signed in"],
  [`${live.apply.files_written}/${live.review.files}`, "reviewed files applied"],
  [`${live.fixture_tests_after_apply.passed}/${live.fixture_tests_after_apply.passed + live.fixture_tests_after_apply.failed}`, "project tests passing after Apply"],
] as const;

function Pane({ title, meta, stats, testId }: { title: string; meta: string; stats: readonly (readonly [string, string])[]; testId: string }) {
  return <div className="relative rounded-md border border-white/15 p-2 pt-4" data-testid={testId}>
    <span className="absolute -top-2.5 left-4 bg-background px-2 font-mono text-xs text-[#7d7d87]"><b className="font-semibold uppercase tracking-[0.12em] text-[#f5f5f7]">{title}</b> · {meta}</span>
    <dl className="grid grid-cols-2 gap-px overflow-hidden rounded-[4px] bg-white/10">
      {stats.map(([value, label]) => <div key={label} className="flex flex-col-reverse bg-[#09090b] p-5 sm:p-6">
        <dt className="mt-2 font-mono text-xs leading-snug text-[#aaaab3]">{label}</dt>
        <dd className="m-0 text-3xl font-semibold tabular-nums tracking-[-0.03em] text-[#f5f5f7] sm:text-4xl">{value}</dd>
      </div>)}
    </dl>
  </div>;
}

export function ProofSection() {
  return <section className="border-b border-white/10" aria-labelledby="proof-title" data-testid="proof-section">
    <div className="mx-auto max-w-[92rem] px-4 py-[clamp(4.5rem,7vw,7.5rem)] sm:px-6 lg:px-10">
      <div className="grid gap-10 lg:grid-cols-[0.8fr_1.4fr]">
        <div>
          <h2 id="proof-title" className="text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">Measured, with the limits printed.</h2>
          <p className="mt-6 max-w-[44ch] text-base leading-relaxed text-[#b4b4bd]">Pybench builds throwaway repositories where agent branches each pass their own checks but break together, and compares what per-branch CI would put on main with what <code className="font-mono text-[0.92em]">pytxo land</code> decides. It shows the mechanism, not how often it happens in real projects. Run it yourself with <code className="font-mono text-[0.92em]">pytxo bench</code>.</p>
          <p className="mt-4 max-w-[44ch] text-sm leading-relaxed text-[#8d8d96]">The live run used the released 1.2.4 Desktop with real Claude Code, Codex and Cursor Agent sessions in {minutes(live.run_seconds)}. Its checks passed and the applied app still had integration bugs the agents&apos; tests missed. Checks passing is evidence, not proof.</p>
          <div className="mt-6 flex flex-wrap gap-x-6 gap-y-2 text-sm">
            <Link href="/bench" className="aperture-link">Pybench results →</Link>
            <Link href="/docs/evidence" className="aperture-link">All evidence and limits →</Link>
          </div>
        </div>
        <div className="grid gap-8 self-start">
          <Pane title="pybench · land" meta={`pytxo ${bench.pytxo_version} · ${bench.machine.os}`} stats={BENCH_STATS} testId="proof-bench" />
          <Pane title="live run" meta={`${live.recorded} · ${minutes(live.run_seconds)}`} stats={LIVE_STATS} testId="proof-live" />
        </div>
      </div>
    </div>
  </section>;
}
