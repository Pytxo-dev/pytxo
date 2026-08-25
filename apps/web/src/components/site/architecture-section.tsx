import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

type Stage = {
  label: string;
  detail: string;
};

const PIPELINE: Stage[] = [
  { label: "Editor or terminal", detail: "Cursor, VS Code, or shell" },
  { label: "MCP hub", detail: "Optional local IDE bridge" },
  { label: "Orchestration", detail: "Rust: projects, permissions, waves" },
  { label: "Execution yard", detail: "Claude Code, Codex, and other CLIs" },
];

const DESKTOP: Stage = {
  label: "Pytxo Desktop",
  detail: "Ops, Missions, Run Review, and recovery",
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
  return (
    <div
      className={cn(
        "flex flex-col gap-2 rounded-[4px] border p-4",
        emphasized ? "border-border bg-card" : "border-border bg-background",
      )}
    >
      <div className="flex items-center justify-between gap-2">
        <p className="text-sm font-medium">{stage.label}</p>
        {optional ? (
          <Badge variant="outline" className="border-border text-[10px] text-muted-foreground">
            Optional
          </Badge>
        ) : null}
      </div>
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
            return (
              <li
                key={stage.label}
                className="flex flex-col gap-1 rounded-[4px] border border-border bg-background p-4"
              >
                <p className="text-sm font-medium">{stage.label}</p>
                <p className="text-xs text-muted-foreground">{stage.detail}</p>
              </li>
            );
          })}
          <li className="ml-4 flex flex-col gap-1 rounded-[4px] border border-border bg-card p-4">
            <div className="flex items-center gap-2">
              <p className="text-sm font-medium">{DESKTOP.label}</p>
              <Badge variant="outline" className="border-border text-[10px] text-muted-foreground">
                Optional
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground">{DESKTOP.detail}</p>
          </li>
        </ol>
      </div>
    </section>
  );
}
