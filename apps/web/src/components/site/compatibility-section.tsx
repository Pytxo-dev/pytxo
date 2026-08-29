/**
 * Mirrors `REGISTRY` in `crates/pytxo-core/src/ade_registry.rs`. Each row is a
 * real probe binary and the real default command Pytxo runs, so the claim is
 * checkable rather than a logo wall.
 */
const REGISTRY = [
  { name: "Claude Code", bin: "claude", cmd: "claude -p" },
  { name: "Antigravity", bin: "agy", cmd: "agy" },
  { name: "OpenAI Codex", bin: "codex", cmd: "codex exec --sandbox workspace-write" },
  { name: "Cursor Agent", bin: "cursor-agent", cmd: "cursor-agent -p --trust" },
  { name: "OpenCode", bin: "opencode", cmd: "opencode run" },
  { name: "Gemini CLI", bin: "gemini", cmd: "gemini --skip-trust -p" },
  { name: "GitHub Copilot CLI", bin: "copilot", cmd: "copilot -p" },
  { name: "Aider", bin: "aider", cmd: "aider --message" },
] as const;

export function CompatibilitySection() {
  return (
    <section
      className="section-pad mx-auto max-w-[92rem] lg:px-10"
      aria-labelledby="compatibility-title"
      data-testid="compatibility-section"
    >
      <div className="flex flex-wrap items-end justify-between gap-8">
        <div className="max-w-[40rem]">
          <h2
            id="compatibility-title"
            className="text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]"
          >
            It runs the agents you already installed.
          </h2>
          <p className="mt-7 text-base leading-relaxed text-[#a9a9b2] sm:text-lg">
            Pytxo detects these CLIs by probing for their binary on your PATH. There is
            no Pytxo model, no provider account, and no proxy in the default path.
          </p>
        </div>
        <p className="max-w-[24rem] text-sm leading-relaxed text-[#7d7d87]">
          Anything you can run in a terminal also works through{" "}
          <code className="font-mono text-[#c7c7ce]">pytxo run --cmd</code>. Editors
          connect through the local MCP hub instead.
        </p>
      </div>

      <div className="mt-14 overflow-hidden rounded-[8px] border border-[var(--aperture-line)]">
        <div className="hidden grid-cols-[minmax(0,1fr)_minmax(0,0.7fr)_minmax(0,1.3fr)] gap-6 border-b border-[var(--aperture-line)] bg-[var(--aperture-raised)] px-6 py-3 font-mono text-[11px] uppercase tracking-[0.05em] text-[#6f6f79] sm:grid">
          <span>Agent</span>
          <span>Detected binary</span>
          <span>Default command</span>
        </div>
        {REGISTRY.map((entry) => (
          <div
            key={entry.bin}
            className="grid gap-1 border-b border-[var(--aperture-line)] px-6 py-4 last:border-b-0 sm:grid-cols-[minmax(0,1fr)_minmax(0,0.7fr)_minmax(0,1.3fr)] sm:items-center sm:gap-6"
          >
            <span className="text-[15px] text-[#f5f5f7]">{entry.name}</span>
            <span className="font-mono text-[13px] text-[#a9a9b2]">{entry.bin}</span>
            <span className="truncate font-mono text-[13px] text-[#7d7d87]">{entry.cmd}</span>
          </div>
        ))}
      </div>
    </section>
  );
}
