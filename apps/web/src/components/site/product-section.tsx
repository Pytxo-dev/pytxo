const STEPS = [
  ["01", "Bound the work.", "Describe one useful change. Review its paths, dependencies and verification commands before a worker starts."],
  ["02", "Inspect one candidate.", "See how task outputs come together. Read the exact prepared files and the checks recorded against their combined result."],
  ["03", "Decide what lands.", "Apply is an explicit repository transition. Return to its recorded outcome, or the recovery action it still needs."],
] as const;
export function ProductSection() {
  return <section className="mx-auto max-w-[92rem] px-4 py-16 sm:px-6 lg:px-10" aria-labelledby="product-title" data-testid="product-section">
    <div className="grid gap-8 lg:grid-cols-[1fr_1.2fr]"><h2 id="product-title" className="max-w-[20ch] text-3xl leading-tight tracking-[-0.025em] sm:text-4xl">Useful work.<br />An inspectable result.</h2><p className="max-w-[52ch] text-base leading-relaxed text-[#aaaab3]">Codex does the coding. Pytxo gives the mission a bounded plan, a combined candidate and an explicit integration decision. You keep your editor and your agent account.</p></div>
    <ol className="mt-12 grid gap-8 md:grid-cols-3" data-testid="product-loop">{STEPS.map(([n,title,body])=><li key={n} className="border-t border-white/15 pt-5"><span className="font-mono text-xs text-[#aaaab3]">{n}</span><h3 className="mt-4 text-xl tracking-[-0.02em]">{title}</h3><p className="mt-3 max-w-[38ch] text-sm leading-relaxed text-[#aaaab3]">{body}</p></li>)}</ol>
  </section>;
}
