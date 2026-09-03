import Link from "next/link";

import { StateChip, type StateTone } from "@/components/site/state-chip";

/**
 * The receipt structure here mirrors `PermissionEnforcementReceipt` in
 * `crates/pytxo-core/src/moat/permission/mod.rs`: four surfaces, each carrying
 * one of four statuses and the mechanism that produced it. All four statuses
 * appear so the honest ones are as visible as the reassuring ones.
 */
const SURFACES = [
  {
    surface: "Workspace isolation",
    status: "Enforced",
    tone: "verified" as StateTone,
    mechanism: "git-worktree",
    note: "Each agent writes into its own copy of the repository.",
  },
  {
    surface: "Host filesystem",
    status: "Advisory only",
    tone: "claimed" as StateTone,
    mechanism: "child-cwd-and-policy-gates",
    note: "Pytxo constrains the child process, but the OS is not enforcing it.",
  },
  {
    surface: "Network",
    status: "Unavailable",
    tone: "unknown" as StateTone,
    mechanism: "not-reported-on-this-platform",
    note: "No claim is made. Unknown is not styled as a pass or a warning.",
  },
  {
    surface: "Apply boundary",
    status: "Enforced",
    tone: "verified" as StateTone,
    mechanism: "reviewed-run-atomic-apply",
    note: "Only reviewed bytes are written, to one repository root, journalled.",
  },
] as const;

export function BoundarySection() {
  return (
    <section
      className="border-y border-[var(--aperture-line)] bg-[var(--aperture-raised)]"
      aria-labelledby="boundary-title"
      data-testid="boundary-section"
    >
      <div className="section-pad mx-auto grid max-w-[92rem] gap-14 lg:grid-cols-[0.85fr_1.15fr] lg:gap-20 lg:px-10">
        <div>
          <h2
            id="boundary-title"
            className="max-w-[20ch] text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]"
          >
            The boundary tells you what it could not prove.
          </h2>
          <div className="mt-8 max-w-[40rem] space-y-5 text-base leading-relaxed text-[#a9a9b2]">
            <p>
              Every run records how each isolation surface was actually held. A surface
              is <span className="text-[#f5f5f7]">enforced</span> when the mechanism
              held it, <span className="text-[#f5f5f7]">advisory</span> when Pytxo asked
              but the platform did not guarantee it,{" "}
              <span className="text-[#f5f5f7]">unavailable</span> when nothing was
              measured, and <span className="text-[#f5f5f7]">bypassed</span> when it was
              deliberately turned off.
            </p>
            <p>
              Advisory and unavailable are not rendered as green. A run whose network
              isolation was never measured says so, on the same screen where you decide
              whether to apply it.
            </p>
            <p className="text-[#c7c7ce]">
              This is the whole product in one sentence: you approve an exact package,
              and you approve it knowing the limits of the evidence behind it.
            </p>
          </div>
          <Link
            href="/docs/concepts/what-is-pytxo"
            className="aperture-link mt-9 inline-block text-sm text-[#a9a9b2] transition-colors hover:text-white"
          >
            How the commit boundary works
          </Link>
        </div>

        <div className="aperture-panel overflow-hidden">
          <div className="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--aperture-line)] px-5 py-4 sm:px-7">
            <span className="font-mono text-[12px] uppercase tracking-[0.05em] text-[#7d7d87]">
              Enforcement receipt
            </span>
            <StateChip tone="unknown" label="Enforcement not fully reported" />
          </div>
          <ul>
            {SURFACES.map((row) => (
              <li
                key={row.surface}
                className="border-b border-[var(--aperture-line)] px-5 py-5 last:border-b-0 sm:px-7"
              >
                <div className="flex flex-wrap items-baseline justify-between gap-x-6 gap-y-2">
                  <span className="text-[15px] font-medium text-[#f5f5f7]">{row.surface}</span>
                  <StateChip tone={row.tone} label={row.status} />
                </div>
                <p className="mt-2 font-mono text-[12px] text-[#6f6f79]">{row.mechanism}</p>
                <p className="mt-2 max-w-[46ch] text-sm leading-relaxed text-[#8d8d96]">{row.note}</p>
              </li>
            ))}
          </ul>
          <div className="border-t border-[var(--aperture-line)] bg-[#0d0d0f] px-5 py-4 sm:px-7">
            <p className="text-sm text-[#8d8d96]">
              Orbit and Galaxy hold prepared changes for explicit Apply. DeepSpace is
              non-flushable; Supernova writes directly.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
