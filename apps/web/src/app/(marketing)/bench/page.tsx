import Link from "next/link";
import reference from "../../../../public/evidence/pybench-2.0.1-2026-11-05.json";

import { InstallSnippet } from "@/components/site/install-snippet";
import s from "@/components/site/site.module.css";
import { linkUrl } from "@/lib/link";
import { pageMetadata } from "@/lib/page-metadata";
import { cn } from "@/lib/utils";

export const metadata = pageMetadata(
  "/bench",
  "Pybench results · Pytxo",
  "Offline benchmarks of the check: what per-branch CI would land compared with what the check decides, what the check costs, and the signal suite's scaffold size.",
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

const sec = (ms: number) => `${(ms / 1000).toFixed(1)} s`;

export default async function BenchPage() {
  const entries = await board();
  const land = reference.land;
  const sum = land.summary;
  return <div className={cn(s.page, "tabular-nums")}>
    <div className={cn(s.wrap, s.inner)}>
      <h1>What lands on main, measured.</h1>
      <p className={s.lede}>Pybench builds a throwaway repository for each scenario and gives two policies the same agent branches. <b>Per-branch CI</b> tests each branch alone and merges every branch that passed. <b>pytxo land</b> tests the combination and refuses what moved. No model is called. The scenarios show the mechanism; they do not say how often each situation happens in real projects.</p>

      <section className={s.part} aria-labelledby="ref-title" data-testid="bench-reference">
        <div className={s.head}>
          <h2 id="ref-title">Reference run</h2>
          <span className={s.dim}>pytxo {reference.pytxo_version} · {reference.machine.os} {reference.machine.arch} · {reference.machine.cpus} cpus · {reference.created_at.slice(0, 10)}</span>
        </div>
        <dl className={s.stats}>
          {[
            ["Broken mains refused", `${sum.pytxo_prevented}/${sum.naive_landed_broken}`],
            ["Decided as expected", `${sum.correct}/${sum.scenarios}`],
            ["Good changes refused", String(sum.false_refusals)],
            ["Median gate time", sec(sum.median_gate_ms)],
          ].map(([label, value]) => <div key={label}>
            <dt>{label}</dt>
            <dd>{value}</dd>
          </div>)}
        </dl>
        <p id="ref-caption" className={s.note}>Each scenario, what per-branch CI would do, and what pytxo land decided. Gate time includes running the checks. Built from commit <code>ebab582</code> and run on one laptop with other work going, so read the timings as rough.</p>
        <div className={cn(s.scroll, "relative overflow-x-auto")}>
          <table className={cn(s.table, "min-w-[44rem]")} aria-describedby="ref-caption">
            <thead><tr>
              <th>Scenario</th><th>Per-branch CI</th><th>pytxo land</th><th className={s.num}>Gate</th>
            </tr></thead>
            <tbody>
              {land.scenarios.map((x) => <tr key={x.id}>
                <th scope="row">{x.title}</th>
                <td>{x.naive_outcome === "landed_broken" && <span className={s.dot} aria-hidden="true" />}{NAIVE[x.naive_outcome] ?? x.naive_outcome}</td>
                <td>{x.pytxo_decision}{x.pytxo_layer && x.pytxo_decision !== "stale" ? ` (${x.pytxo_layer.replace("_", " ")})` : ""}{x.symbol_collisions ? " · same function" : ""}</td>
                <td className={s.num}>{sec(x.gate_ms)}</td>
              </tr>)}
            </tbody>
          </table>
        </div>
        {reference.overhead && <>
          <h3>Gate cost without checks</h3>
          <p className={s.note}>Median of three runs: sequential <code>git merge</code> of every input against <code>pytxo land</code> in shadow mode, which also maps file and function ownership and writes the receipt. Git process start-up dominates both on Windows.</p>
          <div className={cn(s.scroll, "relative overflow-x-auto")}>
            <table className={cn(s.table, "min-w-[28rem]")}>
              <thead><tr><th>Inputs</th><th>Files</th><th className={s.num}>git merge</th><th className={s.num}>pytxo land</th></tr></thead>
              <tbody>
                {reference.overhead.rows.map((r) => <tr key={`${r.inputs}-${r.files}`}><td>{r.inputs}</td><td>{r.files}</td><td className={s.num}>{sec(r.git_merge_ms)}</td><td className={s.num}>{sec(r.pytxo_land_ms)}</td></tr>)}
              </tbody>
            </table>
          </div>
        </>}
        {reference.signal && <p className={s.note}>The signal suite on the Pytxo repository: {reference.signal.files} source files, {reference.signal.original_bytes.toLocaleString("en-US")} bytes reduced to a {reference.signal.scaffold_bytes.toLocaleString("en-US")}-byte structural scaffold ({reference.signal.weighted_reduction_pct}% smaller). Bytes, not tokens or cost.</p>}
        <div className={s.act}>
          <a className={s.quiet} href="/evidence/pybench-2.0.1-2026-11-05.json">Raw results (JSON)</a>
          <Link className={s.quiet} href="/docs/evidence/pybench">Method and limits</Link>
        </div>
      </section>

      <section className={s.part} aria-labelledby="board-title" data-testid="bench-board">
        <h2 id="board-title">Board</h2>
        <p className={s.note}>Summaries published by signed-in users with <code>pytxo bench --submit</code>: machine type and numbers only.</p>
        {entries && entries.length > 0 ? <div className={cn(s.scroll, "relative overflow-x-auto")}>
          <table className={cn(s.table, "min-w-[40rem]")}>
            <thead><tr><th>Date</th><th>Machine</th><th>Pytxo</th><th>Refused / broken</th><th>As expected</th><th className={s.num}>Median gate</th></tr></thead>
            <tbody>
              {entries.map((e) => <tr key={e.id}>
                <td>{e.created_at.slice(0, 10)}</td>
                <td>{e.payload.os} {e.payload.arch} · {e.payload.cpus} cpus</td>
                <td>{e.payload.pytxo_version}</td>
                <td>{e.payload.pytxo_prevented}/{e.payload.naive_landed_broken}</td>
                <td>{e.payload.correct}/{e.payload.scenarios}</td>
                <td className={s.num}>{sec(e.payload.median_gate_ms)}</td>
              </tr>)}
            </tbody>
          </table>
        </div> : <p className={s.note}>{entries ? "No published results yet. Run it locally:" : "The board is unavailable right now. Run it locally:"}</p>}
        <InstallSnippet label="Pybench" copyLabel="Copy the Pybench command">npx pytxo bench</InstallSnippet>
        <p className={s.note}>Needs git and Node.js; nothing leaves your machine. Publishing a result to this board (<code>--submit</code>, after you <Link href="/docs/getting-started/account">sign in</Link>) needs a full source build of Pytxo.</p>
      </section>
    </div>
  </div>;
}
