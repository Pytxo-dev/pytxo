import Link from "next/link";
import reference from "../../../../public/evidence/pybench-1.3.0-2026-10-10.json";

import { InstallSnippet } from "@/components/site/install-snippet";
import { linkUrl } from "@/lib/link";
import { pageMetadata } from "@/lib/page-metadata";

export const metadata = pageMetadata(
  "/bench",
  "Pybench results · Pytxo",
  "Offline benchmarks of the Pytxo landing gate: what per-branch CI would land compared with what pytxo land decides, gate cost, and Signal Core scaffold size.",
);
export const revalidate = 300;

type BoardEntry = {
  id: string;
  created_at: string;
  payload: {
    pytxo_version: string;
    os: string;
    arch: string;
    cpus: number;
    scenarios: number;
    correct: number;
    naive_landed_broken: number;
    pytxo_prevented: number;
    false_refusals: number;
    median_gate_ms: number;
    signal_reduction_pct?: number | null;
  };
};

async function board(): Promise<BoardEntry[] | null> {
  const target = linkUrl("/v1/bench");
  if (!target) return null;
  try {
    const res = await fetch(target, { next: { revalidate: 300 }, signal: AbortSignal.timeout(5_000) });
    return res.ok ? ((await res.json()) as BoardEntry[]) : null;
  } catch {
    return null;
  }
}

const NAIVE: Record<string, string> = {
  landed_ok: "lands, fine",
  landed_broken: "lands a broken main",
  manual_conflict: "stops on a conflict",
  not_modeled: "not modeled",
};

const s = (ms: number) => `${(ms / 1000).toFixed(1)} s`;

export default async function BenchPage() {
  const entries = await board();
  const land = reference.land;
  const sum = land.summary;
  return <div className="mx-auto max-w-[76rem] px-4 py-16 sm:px-6 sm:py-24 lg:px-10">
    <p className="font-mono text-xs uppercase tracking-[0.14em] text-[#8d8d96]">pybench · pytxo bench</p>
    <h1 className="mt-4 max-w-[22ch] text-[clamp(2.25rem,4vw,3.75rem)] leading-[1.04] tracking-[-0.045em]">What lands on main, measured.</h1>
    <p className="mt-6 max-w-[48rem] text-base leading-relaxed text-[#a9a9b2] sm:text-lg">Pybench builds a throwaway repository for each scenario and gives two policies the same agent branches. <b className="font-medium text-[#f5f5f7]">Per-branch CI</b> tests each branch alone and merges every branch that passed. <b className="font-medium text-[#f5f5f7]">pytxo land</b> tests the combination and refuses what moved. No model is called. The scenarios show the mechanism; they do not say how often each situation happens in real projects.</p>

    <section className="mt-14 border-t border-white/10 pt-10" aria-labelledby="ref-title" data-testid="bench-reference">
      <div className="flex flex-wrap items-baseline justify-between gap-4">
        <h2 id="ref-title" className="text-2xl tracking-[-0.025em]">Reference run</h2>
        <p className="font-mono text-xs text-[#8d8d96]">pytxo {reference.pytxo_version} · {reference.machine.os} {reference.machine.arch} · {reference.machine.cpus} cpus · {reference.created_at.slice(0, 10)}</p>
      </div>
      <dl className="mt-8 grid grid-cols-2 gap-8 lg:grid-cols-4">
        {[
          ["Broken mains refused", `${sum.pytxo_prevented}/${sum.naive_landed_broken}`],
          ["Decided as expected", `${sum.correct}/${sum.scenarios}`],
          ["Good changes refused", String(sum.false_refusals)],
          ["Median gate time", s(sum.median_gate_ms)],
        ].map(([label, value]) => <div key={label}>
          <dt className="font-mono text-xs text-[#8d8d96]">{label}</dt>
          <dd className="mt-3 text-3xl tabular-nums text-[#f5f5f7]">{value}</dd>
        </div>)}
      </dl>
      <div className="mt-10 overflow-x-auto">
        <table className="w-full min-w-[44rem] border-collapse text-left text-sm">
          <caption className="mb-4 text-left text-[#8d8d96]">Each scenario, what per-branch CI would do, and what pytxo land decided. Gate time includes running the checks.</caption>
          <thead className="text-[#f5f5f7]"><tr className="border-b border-white/15">
            <th className="py-3 pr-6 font-medium">Scenario</th><th className="py-3 pr-6 font-medium">Per-branch CI</th><th className="py-3 pr-6 font-medium">pytxo land</th><th className="py-3 text-right font-medium">Gate</th>
          </tr></thead>
          <tbody className="text-[#a9a9b2]">
            {land.scenarios.map((x) => <tr key={x.id} className="border-b border-white/10 align-top">
              <th scope="row" className="py-3.5 pr-6 font-normal text-[#c7c7ce]">{x.title}</th>
              <td className={`py-3.5 pr-6 ${x.naive_outcome === "landed_broken" ? "text-[#ff9b8a]" : ""}`}>{NAIVE[x.naive_outcome] ?? x.naive_outcome}</td>
              <td className="py-3.5 pr-6 text-[#f5f5f7]">{x.pytxo_decision}{x.pytxo_layer && x.pytxo_decision !== "stale" ? ` (${x.pytxo_layer.replace("_", " ")})` : ""}{x.symbol_collisions ? " · same function" : ""}</td>
              <td className="py-3.5 text-right font-mono text-xs tabular-nums">{s(x.gate_ms)}</td>
            </tr>)}
          </tbody>
        </table>
      </div>
      {reference.overhead && <div className="mt-12">
        <h3 className="text-lg tracking-[-0.02em]">Gate cost without checks</h3>
        <p className="mt-2 max-w-[48rem] text-sm text-[#8d8d96]">Median of three runs: sequential <code className="font-mono">git merge</code> of every input against <code className="font-mono">pytxo land</code> in shadow mode, which also maps file and function ownership and writes the receipt. Git process start-up dominates both on Windows.</p>
        <div className="mt-5 overflow-x-auto">
          <table className="w-full min-w-[28rem] border-collapse text-left text-sm">
            <thead className="text-[#f5f5f7]"><tr className="border-b border-white/15"><th className="py-2.5 pr-6 font-medium">Inputs</th><th className="py-2.5 pr-6 font-medium">Files</th><th className="py-2.5 pr-6 text-right font-medium">git merge</th><th className="py-2.5 text-right font-medium">pytxo land</th></tr></thead>
            <tbody className="font-mono text-xs text-[#a9a9b2]">
              {reference.overhead.rows.map((r) => <tr key={`${r.inputs}-${r.files}`} className="border-b border-white/10"><td className="py-2.5 pr-6">{r.inputs}</td><td className="py-2.5 pr-6">{r.files}</td><td className="py-2.5 pr-6 text-right">{s(r.git_merge_ms)}</td><td className="py-2.5 text-right">{s(r.pytxo_land_ms)}</td></tr>)}
            </tbody>
          </table>
        </div>
      </div>}
      {reference.signal && <p className="mt-10 max-w-[48rem] text-sm leading-relaxed text-[#8d8d96]">Signal Core on the Pytxo repository: {reference.signal.files} source files, {reference.signal.original_bytes.toLocaleString("en-US")} bytes reduced to a {reference.signal.scaffold_bytes.toLocaleString("en-US")}-byte structural scaffold ({reference.signal.weighted_reduction_pct}% smaller). Bytes, not tokens or cost.</p>}
      <div className="mt-6 flex flex-wrap gap-x-6 gap-y-6 text-sm">
        <a className="aperture-link" href="/evidence/pybench-1.3.0-2026-10-10.json">Raw results (JSON)</a>
        <Link className="aperture-link" href="/docs/evidence/pybench">Method and limits →</Link>
      </div>
    </section>

    <section className="mt-16 border-t border-white/10 pt-10" aria-labelledby="board-title" data-testid="bench-board">
      <h2 id="board-title" className="text-2xl tracking-[-0.025em]">Board</h2>
      <p className="mt-3 max-w-[48rem] text-sm text-[#8d8d96]">Summaries published by signed-in users with <code className="font-mono">pytxo bench --submit</code>: machine type and numbers only.</p>
      {entries && entries.length > 0 ? <div className="mt-6 overflow-x-auto">
        <table className="w-full min-w-[40rem] border-collapse text-left text-sm">
          <thead className="text-[#f5f5f7]"><tr className="border-b border-white/15"><th className="py-2.5 pr-6 font-medium">Date</th><th className="py-2.5 pr-6 font-medium">Machine</th><th className="py-2.5 pr-6 font-medium">Pytxo</th><th className="py-2.5 pr-6 font-medium">Refused / broken</th><th className="py-2.5 pr-6 font-medium">As expected</th><th className="py-2.5 text-right font-medium">Median gate</th></tr></thead>
          <tbody className="text-[#a9a9b2]">
            {entries.map((e) => <tr key={e.id} className="border-b border-white/10">
              <td className="py-2.5 pr-6 font-mono text-xs">{e.created_at.slice(0, 10)}</td>
              <td className="py-2.5 pr-6">{e.payload.os} {e.payload.arch} · {e.payload.cpus} cpus</td>
              <td className="py-2.5 pr-6 font-mono text-xs">{e.payload.pytxo_version}</td>
              <td className="py-2.5 pr-6 tabular-nums">{e.payload.pytxo_prevented}/{e.payload.naive_landed_broken}</td>
              <td className="py-2.5 pr-6 tabular-nums">{e.payload.correct}/{e.payload.scenarios}</td>
              <td className="py-2.5 text-right font-mono text-xs tabular-nums">{s(e.payload.median_gate_ms)}</td>
            </tr>)}
          </tbody>
        </table>
      </div> : <p className="mt-6 text-sm text-[#a9a9b2]">{entries ? "No published results yet. Be the first:" : "The board is unavailable right now. Run it locally:"}</p>}
      <div className="mt-6 max-w-xl">
        <InstallSnippet copyLabel="Copy the Pybench command">pytxo bench --submit</InstallSnippet>
      </div>
      <p className="mt-3 text-sm text-[#8d8d96]">Needs Pytxo 1.3, git and node. <code className="font-mono">--submit</code> asks you to <Link href="/docs/getting-started/account" className="aperture-link">sign in</Link>; without it, nothing leaves your machine.</p>
    </section>
  </div>;
}
