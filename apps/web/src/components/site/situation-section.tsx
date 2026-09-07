import { StateChip } from "@/components/site/state-chip";

/**
 * The problem is a file-level collision, so it is told with the artifact the
 * collision actually produces: two tasks claiming overlapping paths. No
 * invented dashboard, no abstract illustration.
 */
const CLAIMS = [
  { task: "api", paths: "crates/pytxo-orchestrate/src/lib.rs", wave: 1 },
  { task: "ui", paths: "apps/desktop/src/components/desktop2/*", wave: 1 },
  { task: "tests", paths: "crates/pytxo-orchestrate/src/lib.rs", wave: 2 },
] as const;

export function SituationSection() {
  return (
    <section className="section-pad mx-auto max-w-[92rem] lg:px-10" aria-labelledby="situation-title">
      <div className="grid gap-14 lg:grid-cols-[0.9fr_1.1fr] lg:gap-20">
        <div>
          <h2
            id="situation-title"
            className="max-w-[22ch] text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]"
          >
            One agent first. More when it helps.
          </h2>
          <div className="mt-8 max-w-[42rem] space-y-5 text-base leading-relaxed text-[#a9a9b2]">
            <p>
              Start with a bug you can reproduce and a regression test you can run.
              Name the affected files, inspect the plan, and let your existing CLI
              work in its own workspace. Review the prepared result before Apply.
            </p>
            <p>
              When a job has independent parts, the same plan can use several
              instances of the selected CLI. Ownership and dependencies
              stay visible; adding workers does not automatically make a job faster.
            </p>
            <p className="text-[#c7c7ce]">
              In Orbit and Galaxy, Pytxo prepares repository changes for explicit
              Apply. Host filesystem and network controls can still be advisory;
              the enforcement receipt tells you which boundaries were enforced.
            </p>
          </div>
        </div>

        <div className="aperture-panel h-fit overflow-hidden">
          <div className="flex items-center justify-between border-b border-[var(--aperture-line)] px-5 py-3">
            <span className="font-mono text-[12px] uppercase tracking-[0.05em] text-[#7d7d87]">
              Illustrative path claims
            </span>
            <span className="font-mono text-[12px] text-[#7d7d87]">2 waves</span>
          </div>
          <div className="grid grid-cols-[3.5rem_minmax(0,1fr)_4.5rem] gap-4 border-b border-[var(--aperture-line)] px-5 py-2 font-mono text-[11px] uppercase tracking-[0.04em] text-[#6f6f79]">
            <span>Task</span>
            <span>Claimed paths</span>
            <span className="text-right">Wave</span>
          </div>
          {CLAIMS.map((claim) => (
            <div
              key={claim.task}
              className="grid grid-cols-[3.5rem_minmax(0,1fr)_4.5rem] items-center gap-4 border-b border-[var(--aperture-line)] px-5 py-3 last:border-b-0"
            >
              <span className="text-[13px] font-medium text-[#f5f5f7]">{claim.task}</span>
              <span className="truncate font-mono text-[12px] text-[#a9a9b2]">{claim.paths}</span>
              <span className="text-right font-mono text-[13px] tabular-nums text-[#c7c7ce]">
                {claim.wave}
              </span>
            </div>
          ))}
          <div className="flex flex-wrap items-center gap-x-6 gap-y-3 border-t border-[var(--aperture-line)] bg-[var(--aperture-raised)] px-5 py-4">
            <StateChip tone="refuted" label="api and tests overlap" />
            <span className="text-[13px] text-[#7d7d87]">
              so <span className="font-mono text-[#c7c7ce]">tests</span> is scheduled in wave 2,
              never alongside <span className="font-mono text-[#c7c7ce]">api</span>.
            </span>
          </div>
        </div>
      </div>
    </section>
  );
}
