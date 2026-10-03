// Category-level comparison: what each approach hands you, not claims about any one product.
const ROWS = [
  ["You hand out", "One task per agent", "One request. Pytxo plans the split"],
  ["Shared files", "Overlap surfaces when you merge", "A task that shares a file waits for its owner"],
  ["Checks", "Per branch, wherever you set them up", "Run by Pytxo on every task and on the combined change"],
  ["You review", "One diff per agent", "One change, with the agent behind each file"],
  ["What lands", "Whatever you merge", "Exactly the reviewed files, refused if the project moved"],
] as const;

export function CompareSection() {
  return <section className="border-b border-white/10 bg-[#0b0b0d]" aria-labelledby="compare-title" data-testid="compare-section">
    <div className="mx-auto grid max-w-[92rem] gap-12 px-4 py-[clamp(4.5rem,7vw,7.5rem)] sm:px-6 lg:grid-cols-[0.8fr_1.4fr] lg:px-10">
      <div>
        <h2 id="compare-title" className="max-w-[18ch] text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">Why not just run agents side by side?</h2>
        <p className="mt-6 max-w-[42ch] text-base leading-relaxed text-[#b4b4bd]">Agent workspaces give each agent its own branch and leave the merging to you. That works for separate tasks. Pytxo is for one job that several agents finish together.</p>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full min-w-[36rem] border-collapse text-left text-sm sm:text-base">
          <thead><tr className="border-b border-white/15 text-[#f5f5f7]">
            <th scope="col" className="py-3 pr-4 font-medium"><span className="sr-only">Question</span></th>
            <th scope="col" className="py-3 pr-4 font-medium text-[#aaaab3]">Agents side by side</th>
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
