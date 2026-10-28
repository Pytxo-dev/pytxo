// Category-level comparison: what each approach tests, not claims about any one product.
const ROWS = [
  ["Tests", "Each branch alone, against main", "All the agent branches merged together"],
  ["Finds the break", "After a branch merges and main goes red", "Before you merge anything"],
  ["Runs", "In CI, per pull request", "On your machine, in CI, or when an agent stops"],
  ["Tells you", "Which branch failed its own tests", "Whether the branches break together, a branch was already broken, or a test is flaky"],
] as const;

export function CompareSection() {
  return <section className="border-b border-white/10 bg-[#0b0b0d]" aria-labelledby="compare-title" data-testid="compare-section">
    <div className="mx-auto grid max-w-[92rem] gap-12 px-4 py-[clamp(4.5rem,7vw,7.5rem)] sm:px-6 lg:grid-cols-[0.8fr_1.4fr] lg:px-10">
      <div className="min-w-0">
        <h2 id="compare-title" className="max-w-[18ch] text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">Two green branches can still break main.</h2>
        <p className="mt-6 max-w-[42ch] text-base leading-relaxed text-[#b4b4bd]">CI tests each pull request against the base branch, not against the other open pull requests. A merge queue tests combinations, but at merge time, after review. Parallel-agent desktops leave the combining to you. Pytxo tests the combination as soon as the agents finish.</p>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full min-w-[36rem] border-collapse text-left text-sm sm:text-base">
          <thead><tr className="border-b border-white/15 text-[#f5f5f7]">
            <th scope="col" className="py-3 pr-4 font-medium"><span className="sr-only">Question</span></th>
            <th scope="col" className="py-3 pr-4 font-medium text-[#aaaab3]">Per-branch CI</th>
            <th scope="col" className="py-3 font-medium">Pytxo</th>
          </tr></thead>
          <tbody>
            {ROWS.map(([question, side, pytxo]) => <tr key={question} className="border-b border-white/10 align-top">
              <th scope="row" className="py-4 pr-4 font-normal text-[#8d8d96]">{question}</th>
              <td className="py-4 pr-4 text-[#aaaab3]">{side}</td>
              <td className="py-4 text-[#f5f5f7]">{pytxo}</td>
            </tr>)}
          </tbody>
        </table>
      </div>
    </div>
  </section>;
}
