import Link from "next/link";
export function BoundarySection() {
 return <section className="border-y border-white/10 bg-[#101012]" aria-labelledby="boundary-title" data-testid="boundary-section"><div className="mx-auto grid max-w-[92rem] gap-10 px-4 py-16 sm:px-6 lg:grid-cols-[1fr_1.2fr] lg:px-10">
  <div><h2 id="boundary-title" className="max-w-[20ch] text-3xl leading-tight tracking-[-0.025em] sm:text-4xl">Prepared is not applied.</h2><p className="mt-6 max-w-[48ch] text-base leading-relaxed text-[#c4c4cc]">Finished work waits for your review. Apply writes the exact changes you reviewed. If the relevant source changes, refresh and review again.</p></div>
  <div><p className="text-sm leading-relaxed text-[#aaaab3]">This gate protects repository integration. It does not control every host or network side effect. Check the run’s enforcement receipt to see which protections were actually active.</p><Link href="/docs/getting-started/review-and-apply" className="aperture-link mt-6 inline-block text-sm">How Review and Apply work →</Link></div>
 </div></section>;
}
