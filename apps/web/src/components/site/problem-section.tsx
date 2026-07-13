export function ProblemSection() {
  return (
    <section className="section-pad section-reveal mx-auto max-w-6xl">
      <div className="grid gap-12 lg:grid-cols-[1.2fr_0.8fr] lg:items-end lg:gap-20">
        <div className="flex flex-col gap-4">
          <h2 className="text-3xl sm:text-4xl">More output, not more terminal panes</h2>
          <p className="max-w-xl text-muted-foreground leading-relaxed">
            Many cloud agent products put demos first: lots of terminals in a browser tab, high RAM,
            and vendor credits. Pytxo runs agents locally in the background and records what
            actually changed in your code.
          </p>
          <p className="max-w-xl text-sm text-muted-foreground leading-relaxed">
            Your IDE stays your IDE. Agents work in isolated copies, conflicting paths get separate
            waves, and Cursor can drive runs through a local MCP hub.
          </p>
        </div>

        <div className="flex flex-col gap-6 border-l border-border pl-6">
          <div>
            <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
              Typical cloud agent workspace
            </p>
            <ul className="mt-3 space-y-2 text-sm text-muted-foreground">
              <li>Walls of terminals in a heavy browser UI</li>
              <li>High RAM for demos</li>
              <li>Vendor credits and lock-in</li>
            </ul>
          </div>
          <div>
            <p className="text-xs font-medium uppercase tracking-wide text-primary">Pytxo</p>
            <ul className="mt-3 space-y-2 text-sm text-foreground/90">
              <li>Background agents on your machine</li>
              <li>You pay your LLM providers directly</li>
              <li>Optional Desktop: change map and approvals</li>
            </ul>
          </div>
        </div>
      </div>
    </section>
  );
}
