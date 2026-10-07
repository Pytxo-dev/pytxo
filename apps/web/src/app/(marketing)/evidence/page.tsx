import Link from "next/link";
import nativeObservation from "../../../../public/evidence/astra-native-codex-2026-09-07.json";
import checkpointObservation from "../../../../public/evidence/astra-aperture-native-2026-09-08.json";
import fleetRecord from "../../../../public/evidence/fleet-native-2026-10-02.json";

import { StateChip } from "@/components/site/state-chip";
import { SIGNAL_BENCHMARK, SIGNAL_LANGUAGE_BREAKDOWN } from "@/lib/evidence";
import { pageMetadata } from "@/lib/page-metadata";

export const metadata = pageMetadata(
  "/evidence",
  "Evidence · Pytxo",
  "Recorded native runs, including one request run across five agent CLIs, a direct worktree baseline, and the limits of each measurement.",
);

export default function EvidencePage() {
  return (
    <div className="mx-auto max-w-[76rem] px-4 py-16 sm:px-6 sm:py-24 lg:px-10">
      <h1 className="max-w-[24ch] text-[clamp(2.25rem,4vw,3.75rem)] leading-[1.04] tracking-[-0.045em]">
        Evidence and its limits.
      </h1>
      <p className="mt-7 max-w-[46rem] text-base leading-relaxed text-[#a9a9b2] sm:text-lg">
        A recorded agent workflow and a structural byte measurement answer different
        questions. Here are the observed outcomes, their source records, and the
        limits of what they establish.
      </p>

      <section id="fleet" className="mt-16 scroll-mt-24 border-t border-[var(--aperture-line)] pt-10" aria-labelledby="fleet-title">
        <div className="flex flex-wrap items-center gap-4">
          <h2 id="fleet-title" className="text-2xl tracking-[-0.025em]">One request. Five agent CLIs.</h2>
          <StateChip tone="verified" label="Observed locally" />
        </div>
        <p className="mt-6 max-w-[52rem] text-sm leading-relaxed text-[#a9a9b2]">
          On 2 October 2026 the packaged Windows 1.2.2 candidate planned one six-line request
          into {fleetRecord.plan.tasks} tasks and ran them in {fleetRecord.plan.waves.length} steps
          across OpenAI Codex, Claude Code, Cursor Agent, OpenCode and Antigravity, four at once.
          Every worker exited successfully with its task checks passing, and the combined checks
          passed. Review listed {fleetRecord.review.files} files, each with the CLI that prepared it.
          An unrelated file added after review made Apply refuse with nothing written; after a
          refresh, Apply wrote exactly the {fleetRecord.apply.files_written} reviewed files and the
          project&apos;s tests passed {fleetRecord.fixture_tests_after_apply.passed}/{fleetRecord.fixture_tests_after_apply.passed + fleetRecord.fixture_tests_after_apply.failed}.
        </p>
        <dl className="mt-8 grid grid-cols-2 gap-8 lg:grid-cols-4">
          {[
            ["Planned tasks", fleetRecord.plan.tasks],
            ["Agent CLIs", new Set(fleetRecord.workers.map((worker) => worker.cli)).size],
            ["Combined checks passed", `${fleetRecord.combined_checks.passed}/${fleetRecord.combined_checks.recorded}`],
            ["Files applied", fleetRecord.apply.files_written],
          ].map(([label, value]) => <div key={String(label)}>
            <dt className="font-mono text-xs text-[#8d8d96]">{label}</dt>
            <dd className="mt-3 text-3xl tabular-nums text-[#f5f5f7]">{value}</dd>
          </div>)}
        </dl>
        <div className="mt-10 overflow-x-auto">
          <table className="w-full min-w-[35rem] border-collapse text-left text-sm">
            <caption className="mb-4 text-left text-[#8d8d96]">Each task, the CLI that ran it, and the files it prepared.</caption>
            <thead className="text-[#f5f5f7]"><tr className="border-b border-[var(--aperture-line)]">
              <th className="py-3 pr-6 font-medium">Task</th><th className="py-3 pr-6 font-medium">Agent CLI</th><th className="py-3 pr-6 font-medium">Step</th><th className="py-3 font-medium">Files prepared</th>
            </tr></thead>
            <tbody className="text-[#a9a9b2]">
              {fleetRecord.workers.map((worker) => <tr key={worker.task} className="border-b border-[var(--aperture-line)]">
                <th scope="row" className="py-4 pr-6 font-mono text-xs font-normal text-[#c7c7ce]">{worker.task}</th><td className="py-4 pr-6">{worker.cli}</td><td className="py-4 pr-6 tabular-nums">{worker.wave}</td><td className="py-4 font-mono text-xs">{worker.files_prepared.length ? worker.files_prepared.join(", ") : "None"}</td>
              </tr>)}
            </tbody>
          </table>
        </div>
        <ul className="mt-8 max-w-[52rem] list-disc space-y-2 pl-5 text-sm leading-relaxed text-[#a9a9b2]">
          {fleetRecord.limits.map((limit) => <li key={limit}>{limit}</li>)}
        </ul>
        <p className="mt-5 max-w-[52rem] break-all font-mono text-xs leading-relaxed text-[#8d8d96]">
          Recorded fleet MSI SHA256: {fleetRecord.msi_sha256}
        </p>
        <div className="mt-5 flex flex-wrap gap-x-7 gap-y-3 text-sm text-[#f5f5f7]">
          <a className="aperture-link" href="/evidence/fleet-native-2026-10-02.json">Fleet run record</a>
        </div>
      </section>
      <section className="mt-16 border-t border-[var(--aperture-line)] pt-10" aria-labelledby="workflow-title">
        <div className="flex flex-wrap items-center gap-4">
          <h2 id="workflow-title" className="text-2xl tracking-[-0.025em]">One harness. Three scoped tasks.</h2>
          <StateChip tone="verified" label="Observed locally" />
        </div>
        <p className="mt-6 max-w-[52rem] text-sm leading-relaxed text-[#a9a9b2]">
          On 7 September 2026, the packaged Windows 1.2.2 candidate ran a synthetic
          code, tests and documentation mission through Codex 0.153.4. Two independent
          tasks ran first; the test task followed its implementation dependency.
          Pytxo checked the combined candidate before explicit Review and Apply.
          Test automation operated the native app on a development host.
        </p>
        <dl className="mt-8 grid grid-cols-2 gap-8 lg:grid-cols-4">
          {[
            ["Planned tasks", nativeObservation.planned_tasks],
            ["Ordered waves", nativeObservation.waves],
            ["Matching applied hashes", nativeObservation.changed_files.length],
            ["Post-Apply tests passed", nativeObservation.native_apply.post_apply_tests_passed],
          ].map(([label, value]) => <div key={label}>
            <dt className="font-mono text-xs text-[#8d8d96]">{label}</dt>
            <dd className="mt-3 text-3xl tabular-nums text-[#f5f5f7]">{value}</dd>
          </div>)}
        </dl>
        <div className="mt-10 overflow-x-auto">
          <table className="w-full min-w-[35rem] border-collapse text-left text-sm">
            <caption className="mb-4 text-left text-[#8d8d96]">Successful exploratory outcomes, with failed attempts retained below.</caption>
            <thead className="text-[#f5f5f7]"><tr className="border-b border-[var(--aperture-line)]">
              <th className="py-3 pr-6 font-medium">Observed outcome</th><th className="py-3 pr-6 font-medium">Direct Codex + worktree</th><th className="py-3 font-medium">Codex through Pytxo</th>
            </tr></thead>
            <tbody className="text-[#a9a9b2]">
              {[
                ["Requested three-file change", "Completed", "Completed"],
                ["Primary source preserved while workers ran", "Yes", "Yes"],
                ["Independent checks after work or Apply", "Passed", "Passed"],
                ["Frozen package and combined-check receipt", "Not part of this path", "Recorded"],
                ["Explicit native Apply and committed journal", "Not part of this path", "Recorded"],
              ].map(([outcome, direct, pytxo]) => <tr key={outcome} className="border-b border-[var(--aperture-line)]">
                <th scope="row" className="py-4 pr-6 font-normal text-[#c7c7ce]">{outcome}</th><td className="py-4 pr-6">{direct}</td><td className="py-4">{pytxo}</td>
              </tr>)}
            </tbody>
          </table>
        </div>
        <p className="mt-8 max-w-[52rem] text-sm leading-relaxed text-[#a9a9b2]">
          An earlier native attempt was refused because a worker edited a file outside
          its ownership, despite passing task checks. Another stopped before model
          work because Windows split a prompt argument. A third passed checks, but
          review withheld Apply because its README contradicted the changed code.
          These outcomes are retained. The successful run followed the task-handoff
          and prompt-transport repairs and explicit final-state guidance added to
          the documentation task in the native plan editor.
          Different instance counts, startup overhead and background load prevent a
          controlled speed comparison; the direct run also received no task-prompt
          override. Cost and customer repeat use were not measured.
          Host filesystem and network controls remained advisory; extraction and
          execution on this host do not prove a clean Windows installation.
        </p>
        <p className="mt-5 max-w-[52rem] break-all font-mono text-xs leading-relaxed text-[#8d8d96]">
          Recorded MSI SHA256: {nativeObservation.msi_sha256}
        </p>
        <div className="mt-7 flex flex-wrap gap-x-7 gap-y-3 text-sm text-[#f5f5f7]">
          <a className="aperture-link" href="/evidence/astra-native-codex-2026-09-07.json">Native run record</a>
          <a className="aperture-link" href="/evidence/astra-direct-codex-2026-09-07.json">Direct worktree record</a>
          <a className="aperture-link" href="/evidence/astra-refused-run-2026-09-07.json">Earlier refusal record</a>
          <a className="aperture-link" href="/evidence/astra-transport-failure-2026-09-07.json">Launch failure record</a>
          <a className="aperture-link" href="/evidence/astra-review-withheld-2026-09-07.json">Review withheld record</a>
        </div>
        <div className="mt-12 border-t border-[var(--aperture-line)] pt-8">
          <h3 className="text-xl tracking-[-0.025em]">September 8 recorded checkpoint</h3>
          <p className="mt-5 max-w-[52rem] text-sm leading-relaxed text-[#a9a9b2]">
            On 8 September 2026, the MSI with the review-scrolling and reduced-motion
            fixes completed a fresh credentials-path rehearsal. Three
            tasks passed combined checks and explicit native Apply. All three file
            hashes matched, {checkpointObservation.apply.repository_tests_passed} repository
            tests passed, and {checkpointObservation.apply.independent_tests_passed} separately
            authored acceptance checks passed. Five of those acceptance checks had
            failed on the baseline. The committed receipt survived a native restart.
            Cancel left the primary inventory and file contents unchanged.
            Native recordings retain the actual review and Apply interactions;
            removed review waiting time is labeled. This recording predates the
            newer Desktop polish build, whose native verification is pending.
            Clean-install and public-download acceptance remain open. This host
            rehearsal is not a direct-run comparison.
          </p>
          <p className="mt-5 max-w-[52rem] break-all font-mono text-xs leading-relaxed text-[#8d8d96]">
            Recorded checkpoint MSI SHA256: {checkpointObservation.msi_sha256}
          </p>
          <div className="mt-5 flex flex-wrap gap-x-7 gap-y-3 text-sm text-[#f5f5f7]">
            <a className="aperture-link" href="/evidence/astra-aperture-native-2026-09-08.json">September 8 checkpoint record</a>
            <a className="aperture-link" href="/evidence/astra-final-native-2026-09-08.json">Earlier onboarding record</a>
            <a className="aperture-link" href="/evidence/astra-ci-native-2026-09-07.json">Earlier CI candidate record</a>
          </div>
        </div>
      </section>

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
          <div aria-hidden="true" className="hidden grid-cols-[minmax(0,1fr)_5rem_minmax(0,0.9fr)_minmax(0,0.9fr)_6rem] gap-6 border-b border-[var(--aperture-line)] bg-[var(--aperture-raised)] px-6 py-3 font-mono text-[11px] uppercase tracking-[0.05em] text-[#8d8d96] sm:grid">
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
              <h3 className="mb-3 text-[15px] text-[#f5f5f7] sm:mb-0">{row.language}</h3>
              <dl className="space-y-2 sm:contents sm:space-y-0">
                {[
                  ["Files", row.files],
                  ["Source bytes", row.originalBytes.toLocaleString("en-US")],
                  ["Scaffold bytes", row.scaffoldedBytes.toLocaleString("en-US")],
                  ["Reduction", `${row.reductionPct}%`],
                ].map(([label, value]) => (
                  <div key={label} className="flex items-baseline justify-between gap-4 sm:block sm:text-right">
                    <dt className="text-[13px] text-[#a9a9b2] sm:sr-only">{label}</dt>
                    <dd className="font-mono text-[13px] tabular-nums text-[#c7c7ce]">{value}</dd>
                  </div>
                ))}
              </dl>
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
          Measurement record
        </h2>
        <p className="mt-6 max-w-[52rem] text-sm leading-relaxed text-[#a9a9b2]">
          The raw result, including the per-file table for every one of the{" "}
          {SIGNAL_BENCHMARK.files} files, is committed in the public source repository.
          This summary is not an independent reproduction of that corpus.
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
