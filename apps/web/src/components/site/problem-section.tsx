import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export function ProblemSection() {
  return (
    <section className="mx-auto max-w-6xl px-4 py-20 sm:px-6 sm:py-24">
      <div className="flex flex-col gap-10">
        <div className="text-center">
          <h2 className="text-3xl font-semibold tracking-tight sm:text-4xl">
            Throughput, not terminal wallpaper
          </h2>
          <p className="mx-auto mt-4 max-w-2xl text-muted-foreground">
            Cloud agent environments optimize for demos — sixteen panes in a browser tab,
            heavy RAM, proprietary credits. Pytxo coordinates headless agents locally and
            logs structure.
          </p>
        </div>

        <Alert className="glass-panel border-white/10">
          <AlertTitle>Local-first by design</AlertTitle>
          <AlertDescription>
            Your IDE stays your IDE. Pytxo runs PTY-backed agents in isolated copies, schedules
            conflict-aware waves, and exposes MCP tools — no cloud workspace required.
          </AlertDescription>
        </Alert>

        <div className="grid gap-5 md:grid-cols-2">
          <Card className="glass-panel border-white/8">
            <CardHeader>
              <CardTitle className="text-base text-muted-foreground">
                Cloud ADE pattern
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm text-muted-foreground">
              <p>Multi-pane terminal grids in remote containers</p>
              <p>GPU-heavy browser rendering</p>
              <p>Vendor credit models and lock-in</p>
              <p>Telemetry = streamed terminal text</p>
            </CardContent>
          </Card>
          <Card className="glass-panel chroma-edge-top border-white/8">
            <CardHeader>
              <CardTitle className="text-base">Pytxo pattern</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm text-muted-foreground">
              <p>Headless agents in local PTYs + isolated copies</p>
              <p>Rust orchestrator, SQLite WAL telemetry</p>
              <p>BYOK — never the LLM vendor</p>
              <p>Reality Deck: structural graph, approvals, fleet panel</p>
            </CardContent>
          </Card>
        </div>
      </div>
    </section>
  );
}
