import Link from "next/link";

const LOOP = [
  {
    step: "plan",
    title: "Claim paths",
    body: "Assign ownership and dependencies before dispatch so overlapping work waits in a later wave.",
  },
  {
    step: "live",
    title: "Isolate the run",
    body: "Agents execute in isolated copies. Vendor CLIs keep their own sessions; Pytxo coordinates the yard.",
  },
  {
    step: "review",
    title: "Inspect one package",
    body: "Run Review shows the digest, exact file bytes, profile, and path counts for a single repository root.",
  },
  {
    step: "apply",
    title: "Apply or discard",
    body: "Flush the reviewed package, or throw it away. Nothing lands in the real tree until you say so.",
  },
] as const;

const ALSO_INCLUDED = [
  {
    title: "Mission planning",
    description:
      "Compose a mission, review tasks, paths, permissions, agent assignments, and waves, then dispatch the approved plan.",
    href: "/docs/concepts/desktop#new-mission",
  },
  {
    title: "Missions and Run Review",
    description:
      "Track active work and history in Missions. Run Review shows the package, exact file changes, policy evidence, recovery state, and Apply history.",
    href: "/docs/concepts/desktop",
  },
  {
    title: "Your agents keep their sessions",
    description:
      "Codex, Claude Code, Cursor, Gemini, OpenCode, and generic commands stay vendor-owned while Pytxo coordinates execution.",
    href: "/docs/reference/providers-byok",
  },
  {
    title: "Workspaces and fleet runs",
    description:
      "Coordinate folders for planning and execution, or order separate repositories in a fleet. Each repository root keeps its own Apply boundary.",
    href: "/docs/concepts/modular-projects",
  },
] as const;

export function FeatureGrid() {
  return (
    <section className="section-pad mx-auto max-w-6xl">
      <div className="flex max-w-2xl flex-col gap-3">
        <h2 className="text-3xl sm:text-4xl">plan | live | review | apply</h2>
        <p className="text-muted-foreground">
          Pytxo coordinates mixed CLIs around one repository root: claim paths, isolate
          writes, review one package, then Apply or discard.
        </p>
      </div>

      <ol className="mt-12 grid gap-px overflow-hidden rounded-lg border border-border bg-border sm:grid-cols-4">
        {LOOP.map((item) => (
          <li key={item.step} className="flex flex-col gap-3 bg-background p-5 sm:p-6">
            <p className="font-mono text-xs uppercase tracking-[0.14em] text-muted-foreground">
              {item.step}
            </p>
            <h3 className="text-base font-semibold tracking-tight">{item.title}</h3>
            <p className="text-sm leading-relaxed text-muted-foreground">{item.body}</p>
          </li>
        ))}
      </ol>

      <aside className="apply-receipt mt-8 max-w-xl" aria-label="Apply receipt">
        <p className="apply-receipt__apply">Apply receipt</p>
        <dl>
          <dt>Digest</dt>
          <dd>pkg-71ad-immutable</dd>
          <dt>Root</dt>
          <dd>/work/acme</dd>
          <dt>Profile</dt>
          <dd>orbit</dd>
          <dt>Paths</dt>
          <dd>12 claimed · 3 waiting</dd>
          <dt>State</dt>
          <dd>waiting</dd>
        </dl>
      </aside>

      <div className="mt-16 flex max-w-2xl flex-col gap-3">
        <h3 className="text-2xl font-semibold tracking-tight">
          Fits the tools you already use
        </h3>
        <p className="text-muted-foreground">
          Keep your editor, agent accounts, and provider keys. Add the coordination layer
          around them.
        </p>
      </div>
      <div className="mt-8 grid gap-px overflow-hidden rounded-lg border border-border bg-border sm:grid-cols-2">
        {ALSO_INCLUDED.map((item) => (
          <div key={item.title} className="flex h-full flex-col gap-3 bg-background p-6">
            <h4 className="font-semibold tracking-tight">{item.title}</h4>
            <p className="text-sm leading-relaxed text-muted-foreground">{item.description}</p>
            <Link
              href={item.href}
              className="mt-auto text-sm font-medium text-primary transition-colors hover:text-foreground"
            >
              Learn more
            </Link>
          </div>
        ))}
      </div>
    </section>
  );
}
