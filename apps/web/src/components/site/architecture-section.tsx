const STEPS = [
  { label: "Editor / terminal", detail: "Cursor, VS Code, or shell" },
  { label: "MCP hub", detail: "Optional IDE bridge" },
  { label: "Coordinator", detail: "Projects, permissions, catalog" },
  { label: "Scheduler", detail: "Waves and fleet order" },
  { label: "Agent processes", detail: "Claude Code, Codex, CLIs" },
  { label: "Desktop", detail: "Topology and approvals" },
] as const;

export function ArchitectureSection() {
  return (
    <section className="border-y border-border bg-card/10">
      <div className="section-pad mx-auto max-w-6xl section-reveal">
        <h2 className="max-w-2xl text-3xl sm:text-4xl">
          Local control plane, not a browser full of terminals
        </h2>
        <p className="mt-4 max-w-2xl text-muted-foreground">
          Pytxo plugs into your existing workflow. It coordinates agents and records what changed.
        </p>

        <ol className="mt-12 flex flex-col gap-0 md:flex-row md:items-stretch md:overflow-x-auto">
          {STEPS.map((step, index) => (
            <li
              key={step.label}
              className="relative flex flex-1 flex-col border-l border-border px-4 py-3 first:border-l-0 first:pl-0 md:border-l-0 md:border-t-0 md:px-3 md:py-0 md:pl-0"
            >
              {index < STEPS.length - 1 ? (
                <span
                  className="pointer-events-none absolute top-5 right-0 hidden h-px w-full bg-border md:block"
                  aria-hidden
                />
              ) : null}
              <p className="relative z-10 font-mono text-xs text-primary">
                {String(index + 1).padStart(2, "0")}
              </p>
              <p className="relative z-10 mt-2 text-sm font-medium">{step.label}</p>
              <p className="relative z-10 mt-1 text-xs text-muted-foreground">{step.detail}</p>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}
