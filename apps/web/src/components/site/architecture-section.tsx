import { Blocks, Cpu, LayoutDashboard, Plug, Terminal, type LucideIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

type Stage = {
  label: string;
  detail: string;
  icon: LucideIcon;
};

const PIPELINE: Stage[] = [
  { label: "Editor or terminal", detail: "Cursor, VS Code, or shell", icon: Terminal },
  { label: "MCP hub", detail: "Optional local IDE bridge", icon: Plug },
  { label: "Orchestration", detail: "Rust: projects, permissions, waves", icon: Cpu },
  { label: "Execution yard", detail: "Claude Code, Codex, and other CLIs", icon: Blocks },
];

const DESKTOP: Stage = {
  label: "Pytxo Desktop",
  detail: "Flow, Run Review, operations, and recovery",
  icon: LayoutDashboard,
};

function PipelineNode({
  stage,
  emphasized = false,
  optional = false,
}: {
  stage: Stage;
  emphasized?: boolean;
  optional?: boolean;
}) {
  const Icon = stage.icon;
  return (
    <div
      className={cn(
        "flex flex-col gap-2 rounded-[var(--radius-lg)] border p-4",
        emphasized ? "border-primary/30 bg-primary/[0.04]" : "border-border bg-background/60",
      )}
    >
      <div className="flex items-center justify-between gap-2">
        <span
          className={cn(
            "flex size-8 items-center justify-center rounded-[var(--radius-md)] border",
            emphasized
              ? "border-primary/30 bg-primary/10 text-primary"
              : "border-border bg-muted/30 text-muted-foreground",
          )}
        >
          <Icon className="size-4" />
        </span>
        {optional ? (
          <Badge variant="outline" className="border-border text-[10px] text-muted-foreground">
            Optional
          </Badge>
        ) : null}
      </div>
      <p className="text-sm font-medium">{stage.label}</p>
      <p className="text-xs text-muted-foreground">{stage.detail}</p>
    </div>
  );
}

function Connector() {
  return (
    <div className="flex h-full items-center justify-center">
      <div className="h-px w-full bg-border" />
    </div>
  );
}

export function ArchitectureSection() {
  return (
    <section className="border-y border-border bg-card/10">
      <div className="section-pad mx-auto max-w-6xl">
        <h2 className="max-w-2xl text-3xl sm:text-4xl">
          One local orchestrator behind every surface
        </h2>
        <p className="mt-4 max-w-2xl text-muted-foreground">
          Desktop and the CLI read the same persisted run contract: plan, permission
          profile, enforcement receipt, prepared package, and Apply state.
        </p>

        <div className="mt-14 hidden lg:grid lg:grid-cols-[1fr_2.5rem_1fr_2.5rem_1fr_2.5rem_1fr] lg:gap-y-3">
          <PipelineNode stage={PIPELINE[0]} />
          <Connector />
          <PipelineNode stage={PIPELINE[1]} />
          <Connector />
          <PipelineNode stage={PIPELINE[2]} emphasized />
          <Connector />
          <PipelineNode stage={PIPELINE[3]} />

          <div className="col-start-5 flex justify-center py-1">
            <div className="h-6 w-px bg-border" />
          </div>

          <div className="col-start-5">
            <PipelineNode stage={DESKTOP} optional />
          </div>
        </div>

        <ol className="mt-12 flex flex-col gap-3 lg:hidden">
          {PIPELINE.map((stage) => {
            const Icon = stage.icon;
            return (
              <li
                key={stage.label}
                className="flex items-center gap-4 rounded-[var(--radius-lg)] border border-border bg-background/60 p-4"
              >
                <span className="flex size-8 shrink-0 items-center justify-center rounded-[var(--radius-md)] border border-border bg-muted/30 text-muted-foreground">
                  <Icon className="size-4" />
                </span>
                <div>
                  <p className="text-sm font-medium">{stage.label}</p>
                  <p className="text-xs text-muted-foreground">{stage.detail}</p>
                </div>
              </li>
            );
          })}
          <li className="ml-4 flex items-center gap-4 rounded-[var(--radius-lg)] border border-primary/30 bg-primary/[0.04] p-4">
            <span className="flex size-8 shrink-0 items-center justify-center rounded-[var(--radius-md)] border border-primary/30 bg-primary/10 text-primary">
              <LayoutDashboard className="size-4" />
            </span>
            <div>
              <div className="flex items-center gap-2">
                <p className="text-sm font-medium">{DESKTOP.label}</p>
                <Badge variant="outline" className="border-border text-[10px] text-muted-foreground">
                  Optional
                </Badge>
              </div>
              <p className="text-xs text-muted-foreground">{DESKTOP.detail}</p>
            </div>
          </li>
        </ol>
      </div>
    </section>
  );
}
