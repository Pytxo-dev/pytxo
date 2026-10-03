import Link from "next/link";
import record from "../../../public/evidence/fleet-native-2026-10-02.json";

const minutes = (seconds: number) => `${Math.floor(seconds / 60)} min ${seconds % 60} s`;
const STATS = [
  [String(record.plan.tasks), "tasks from one request"],
  [String(new Set(record.workers.map((worker) => worker.cli)).size), "agent CLIs in one run"],
  [`${record.combined_checks.passed}/${record.combined_checks.recorded}`, "checks passed on the combined change"],
  [String(record.review.files), "files reviewed, each with its agent"],
  ["1", "Apply refused after a file changed"],
  [`${record.fixture_tests_after_apply.passed}/${record.fixture_tests_after_apply.passed + record.fixture_tests_after_apply.failed}`, "project tests passing after Apply"],
] as const;

export function ProofSection() {
  return <section className="border-b border-white/10" aria-labelledby="proof-title" data-testid="proof-section">
    <div className="mx-auto max-w-[92rem] px-4 py-[clamp(4.5rem,7vw,7.5rem)] sm:px-6 lg:px-10">
      <div className="grid gap-10 lg:grid-cols-[0.8fr_1.4fr]">
        <div>
          <h2 id="proof-title" className="text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">One recorded run.</h2>
          <p className="mt-6 max-w-[44ch] text-base leading-relaxed text-[#b4b4bd]">On 2 October 2026 the packaged Windows candidate ran a six-line request across five agent CLIs in {minutes(record.run_seconds)}. A file added after review made Apply refuse; after a refresh, Apply wrote exactly the {record.apply.files_written} reviewed files.</p>
          <p className="mt-4 max-w-[44ch] text-sm leading-relaxed text-[#8d8d96]">In this run OpenCode and Antigravity finished their tasks without changing files. Both causes are fixed; a repeat run is pending.</p>
          <Link href="/evidence#fleet" className="aperture-link mt-6 inline-block text-sm">Read the run record →</Link>
        </div>
        <dl className="grid grid-cols-2 gap-px overflow-hidden rounded-lg border border-white/10 bg-white/10 sm:grid-cols-3">
          {STATS.map(([value, label]) => <div key={label} className="flex flex-col-reverse bg-[#09090b] p-5 sm:p-6">
            <dt className="mt-2 text-sm leading-snug text-[#aaaab3]">{label}</dt>
            <dd className="m-0 text-3xl font-semibold tabular-nums tracking-[-0.03em] text-[#f5f5f7] sm:text-4xl">{value}</dd>
          </div>)}
        </dl>
      </div>
    </div>
  </section>;
}
