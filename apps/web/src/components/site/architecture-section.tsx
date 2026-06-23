import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

const STEPS = [
  { label: "IDE / CLI", detail: "Cursor, VS Code, terminal" },
  { label: "MCP hub", detail: "Local tool routing" },
  { label: "Hypervisor", detail: "Catalog + domains" },
  { label: "Orchestration", detail: "Waves + fleet DAG" },
  { label: "Execution yard", detail: "Headless PTY agents" },
  { label: "Reality Deck", detail: "Topology & approvals (optional)" },
] as const;

export function ArchitectureSection() {
  return (
    <section className="border-y border-white/5 bg-card/20 backdrop-blur-sm">
      <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6 sm:py-24">
        <h2 className="text-center text-3xl font-semibold tracking-tight sm:text-4xl">
          Control plane, not terminal wallpaper
        </h2>
        <p className="mx-auto mt-4 max-w-2xl text-center text-muted-foreground">
          Pytxo plugs into your existing workflow. It coordinates agents and records
          structure — it does not host sixteen panes in a browser tab.
        </p>
        <div className="mt-12 flex flex-col items-stretch gap-2 sm:flex-row sm:flex-wrap sm:items-center sm:justify-center">
          {STEPS.map((step, index) => (
            <div key={step.label} className="flex items-center gap-2">
              <Card className="glass-panel min-w-[9rem] flex-1 border-white/8 sm:flex-none">
                <CardHeader className="pb-1">
                  <CardTitle className="text-sm font-medium">{step.label}</CardTitle>
                </CardHeader>
                <CardContent className="pt-0 text-xs text-muted-foreground">
                  {step.detail}
                </CardContent>
              </Card>
              {index < STEPS.length - 1 ? (
                <span
                  className="hidden px-1 text-muted-foreground/50 sm:inline"
                  aria-hidden
                >
                  →
                </span>
              ) : null}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
