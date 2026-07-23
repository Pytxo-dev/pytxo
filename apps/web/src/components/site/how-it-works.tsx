import Link from "next/link";

const STEPS = [
  {
    step: "01",
    title: "Install",
    description: "Install the CLI with npm, then run pytxo init and pytxo doctor in your git repo.",
    href: "/docs/getting-started/install",
  },
  {
    step: "02",
    title: "Connect your editor",
    description:
      "Add the local pytxo-mcp server in Cursor or VS Code to start runs from chat, or use the CLI only.",
    href: "/docs/getting-started/mcp-from-cursor",
  },
  {
    step: "03",
    title: "Run a multi-agent job",
    description:
      "Define agents in pytxo.toml, preview the plan, then execute in isolated worktrees.",
    href: "/docs/getting-started/first-three-agent-run",
  },
] as const;

export function HowItWorks() {
  return (
    <section className="border-y border-border bg-card/10">
      <div className="section-pad mx-auto max-w-6xl">
        <div className="max-w-xl">
          <h2 className="text-3xl sm:text-4xl">Up and running in three steps</h2>
          <p className="mt-3 text-muted-foreground">
            No cloud signup. No terminal grid. A local agent hypervisor on your machine.
          </p>
        </div>

        <ol className="mt-12 flex flex-col gap-0 border-t border-border">
          {STEPS.map((item) => (
            <li
              key={item.step}
              className="grid gap-4 border-b border-border py-6 md:grid-cols-[4rem_1fr_auto] md:items-baseline md:gap-8"
            >
              <span className="font-mono text-sm text-primary">{item.step}</span>
              <div>
                <h3 className="text-lg font-semibold tracking-tight">{item.title}</h3>
                <p className="mt-1 max-w-xl text-sm leading-relaxed text-muted-foreground">
                  {item.description}
                </p>
              </div>
              <Link
                href={item.href}
                className="text-sm font-medium text-primary hover:text-foreground md:justify-self-end"
              >
                Guide
              </Link>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}
