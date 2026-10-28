import capture from "../../../public/evidence/demo-blocked-2.0.0-2026-10-28.json";

// The snippets are lines of the recorded demo, not mock output.
const lines = capture.output.split("\n");
const from = (prefix: string, count = 1) => {
  const start = lines.findIndex((line) => line.startsWith(prefix));
  return lines.slice(start, start + count).join("\n");
};

const STEPS = [
  {
    title: "It finds the agent branches.",
    body: "Branches named claude/*, codex/*, copilot/*, cursor/*, devin/*, jules/* or agent/*, and every branch in a linked worktree, with a commit in the last 14 days. Or name the branches yourself.",
    snippet: "$ npx pytxo\n$ npx pytxo feature/a feature/b",
  },
  {
    title: "It merges them and runs your tests.",
    body: "In a throwaway worktree, with the test command it finds in your project: npm test (installing dependencies first when there are any), pytest, cargo test, go test and more. Your repository does not change.",
    snippet: from("Checking"),
  },
  {
    title: "It says what broke and what to do.",
    body: "Three lines: the verdict, why, and what to do next. When the combination fails, each branch runs alone to tell a break that only happens together from a branch that was already broken, and, if every branch passes alone, the combined change runs again to catch a flaky test. If the combined run times out, the branches do not run alone after it.",
    snippet: from("BLOCKED", 3),
  },
] as const;

export function HowItWorks() {
  return <section id="how" className="scroll-mt-20 border-b border-white/10" aria-labelledby="how-title" data-testid="how-it-works">
    <div className="mx-auto max-w-[92rem] px-4 py-[clamp(4.5rem,7vw,7.5rem)] sm:px-6 lg:px-10">
      <h2 id="how-title" className="max-w-[22ch] text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">How it works</h2>
      <ol className="mt-14 grid gap-12 lg:grid-cols-3 lg:gap-10">
        {STEPS.map((step, index) => <li key={step.title} className="flex min-w-0 flex-col">
          <span className="flex items-center gap-3 font-mono text-xs uppercase tracking-[0.14em] text-[#8d8d96]">step 0{index + 1}<i aria-hidden className="h-px w-16 bg-[linear-gradient(90deg,#45dccb,transparent)]" /></span>
          <h3 className="mt-3 text-2xl leading-tight tracking-[-0.03em]">{step.title}</h3>
          <p className="mt-5 max-w-[44ch] text-base leading-relaxed text-[#b4b4bd]">{step.body}</p>
          <div className="mt-auto pt-6">
            <pre className="whitespace-pre-wrap break-words rounded-[4px] border border-white/10 bg-[#09090b] p-4 font-mono text-[11px] leading-[1.65] text-[#c7c7ce]">{step.snippet}</pre>
          </div>
        </li>)}
      </ol>
    </div>
  </section>;
}
