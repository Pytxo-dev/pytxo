import { auth } from "@clerk/nextjs/server";
import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";

import s from "@/components/site/site.module.css";
import { StateChip } from "@/components/site/state-chip";
import { SITE_AUTH_BACKEND_ENABLED } from "@/lib/auth-config";
import { linkUrl } from "@/lib/link";
import { cn } from "@/lib/utils";

type SharedReceipt = {
  id: string;
  kind: "land" | "apply";
  decision: string;
  mode: string;
  created_at: string;
  pytxo_version: string;
  inputs: number;
  agents: string[];
  files_changed: number;
  shared_files: number;
  symbol_collisions: number;
  conflicts: number;
  checks: { combined: { passed: number; total: number }; per_input: { passed: number; total: number } };
  duration_ms: number;
  content_digest: string;
  paths?: string[];
};

const ID = /^r_[0-9A-Za-z]{12}$/;

type Loaded = { created_at: string; payload: SharedReceipt; private?: boolean };

/** Public receipts are cached; a private one (Pytxo Pro) is fetched uncached as its signed-in owner. */
async function load(id: string): Promise<Loaded | null> {
  if (!ID.test(id)) return null;
  const target = linkUrl(`/v1/receipts/${id}`);
  if (!target) return null;
  try {
    const res = await fetch(target, { next: { revalidate: 300 }, signal: AbortSignal.timeout(5_000) });
    if (res.ok) return (await res.json()) as Loaded;
    if (res.status !== 404 || !SITE_AUTH_BACKEND_ENABLED) return null;
    const { userId, getToken } = await auth();
    const token = userId ? await getToken() : null;
    if (!token) return null;
    const own = await fetch(target, {
      headers: { Authorization: `Bearer ${token}` },
      cache: "no-store",
      signal: AbortSignal.timeout(5_000),
    });
    return own.ok ? ((await own.json()) as Loaded) : null;
  } catch {
    return null;
  }
}

const VERDICT: Record<string, { tone: "verified" | "refuted" | "unknown"; label: string; line: string }> = {
  ready: { tone: "verified", label: "Ready", line: "The combined change passed its checks. It was checked in shadow mode, so nothing was landed yet." },
  landed: { tone: "verified", label: "Landed", line: "The combined change passed its checks and was landed onto the branch it was checked against." },
  applied: { tone: "verified", label: "Applied", line: "A reviewed Pytxo run was applied: exactly the reviewed files were written." },
  blocked: { tone: "refuted", label: "Blocked", line: "Pytxo refused to land this combination. Nothing reached the target branch." },
  stale: { tone: "unknown", label: "Stale", line: "The target branch moved while Pytxo was checking, so the result was not landed." },
  refused: { tone: "refuted", label: "Refused", line: "Apply was refused and nothing was written." },
};

export async function generateMetadata(props: { params: Promise<{ id: string }> }): Promise<Metadata> {
  const { id } = await props.params;
  const found = await load(id);
  const title = found ? `Pytxo receipt: ${VERDICT[found.payload.decision]?.label ?? found.payload.decision}` : "Pytxo receipt";
  return { title: { absolute: title }, description: "A shared Pytxo landing receipt.", robots: { index: false, follow: false } };
}

export default async function ReceiptPage(props: { params: Promise<{ id: string }> }) {
  const { id } = await props.params;
  const found = await load(id);
  if (!found) notFound();
  const r = found.payload;
  const verdict = VERDICT[r.decision] ?? { tone: "unknown" as const, label: r.decision, line: "" };
  const rows: [string, string][] = [
    ["Inputs", `${r.inputs}${r.agents.length ? ` · ${r.agents.join(", ")}` : ""}`],
    ["Files changed", String(r.files_changed)],
    ["Checks on the combined change", r.checks.combined.total ? `${r.checks.combined.passed}/${r.checks.combined.total} passed` : "none recorded"],
    ["Checks on each input alone", r.checks.per_input.total ? `${r.checks.per_input.passed}/${r.checks.per_input.total} passed` : "not run"],
    ["Shared files", `${r.shared_files}${r.symbol_collisions ? ` · ${r.symbol_collisions} changed in the same function or type` : ""}`],
    ["Textual conflicts", String(r.conflicts)],
    ["Gate time", `${(r.duration_ms / 1000).toFixed(1)} s`],
    ["Mode", r.mode === "shadow" ? "Shadow (nothing changed)" : "Applied"],
    ["Pytxo", r.pytxo_version],
  ];
  return <div className={s.page} data-testid="shared-receipt">
    <div className={cn(s.wrap, s.inner, "tabular-nums")}>
      <div className={s.head}>
        <h1>{verdict.label}</h1>
        <StateChip tone={verdict.tone} label={r.decision} />
      </div>
      <p className={s.lede}>{r.kind === "land" ? "Landing receipt" : "Apply receipt"} · {r.created_at.slice(0, 10)}. {verdict.line}</p>
      <dl className={s.facts}>
        {rows.map(([term, value]) => <div key={term}>
          <dt>{term}</dt>
          <dd>{value}</dd>
        </div>)}
      </dl>
      {r.paths && r.paths.length > 0 && <details className={s.paths}>
        <summary>{r.paths.length} changed paths</summary>
        <ul>{r.paths.map((p) => <li key={p}>{p}</li>)}</ul>
      </details>}
      <p className={cn(s.note, s.digest)}>Receipt {r.id} · {r.content_digest}</p>
      {found.private && <p className={s.note} data-testid="private-receipt">Private receipt: only you can open this page, signed in.</p>}
      <p className={s.note}>The person who shared this chose to publish counts and outcomes only; commands, branch names, output and the repository stay on their machine. The digest lets them prove this page matches the receipt file they kept.</p>
      <div className={s.act}>
        <Link href="/docs/getting-started/land-branches" className={s.quiet}>What is pytxo land?</Link>
        <Link href="/download" className={s.quiet}>Get Pytxo</Link>
      </div>
    </div>
  </div>;
}
