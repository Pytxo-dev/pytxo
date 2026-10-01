import Link from "next/link";
export function CompatibilitySection() {
  return <section className="mx-auto grid max-w-[92rem] gap-10 px-4 py-16 sm:px-6 lg:grid-cols-2 lg:px-10" aria-labelledby="compatibility-title" data-testid="compatibility-section">
    <div><h2 id="compatibility-title" className="text-3xl tracking-[-0.025em] sm:text-4xl">One repository.<br />Start with one worker.</h2><p className="mt-6 max-w-[44ch] text-sm leading-relaxed text-[#aaaab3]">The v1.2.2 Desktop preview starts with Windows and Codex. Choose a small bug fix with a test you can run. Public downloads are still v1.2.1.</p><Link className="aperture-link mt-6 inline-block text-sm" href="/docs/getting-started/desktop-setup">Check setup requirements →</Link></div>
    <dl className="divide-y divide-white/15 border-y border-white/15">{[["Desktop", "Windows x64"],["Agent", "Codex CLI installed and authenticated"],["Repository", "Git repository with its build and test dependencies"],["Verification", "Commands you approve, checked on the combined candidate"],["Accounts", "No Pytxo account for local Core; agent access is separate"],["Scope", "Repository Apply; not universal host or network control"]].map(([term,value])=><div key={term} className="grid grid-cols-[7rem_1fr] gap-5 py-4 text-sm"><dt className="text-[#aaaab3]">{term}</dt><dd className="leading-relaxed">{value}</dd></div>)}</dl>
  </section>;
}
