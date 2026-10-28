import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { CiSection } from "@/components/site/ci-section";
import { CompareSection } from "@/components/site/compare-section";
import { CompatibilitySection } from "@/components/site/compatibility-section";
import { GetItSection } from "@/components/site/get-it-section";
import { Hero } from "@/components/site/hero";
import { HowItWorks } from "@/components/site/how-it-works";
import { ProofSection } from "@/components/site/proof-section";
import { pageMetadata } from "@/lib/page-metadata";
import { NPM_URL, PRODUCT_LINE, PRODUCT_SUMMARY, PUBLISHED_VERSION, SHARE_IMAGE } from "@/lib/site";

export const metadata = pageMetadata(
  "/",
  "Pytxo: do your agents' branches pass together?",
  PRODUCT_SUMMARY,
);

const STRUCTURED_DATA = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "WebSite",
      "@id": "https://pytxo.com/#website",
      name: "Pytxo",
      url: "https://pytxo.com",
      description: PRODUCT_LINE,
    },
    {
      "@type": "SoftwareApplication",
      "@id": "https://pytxo.com/#software",
      name: "Pytxo",
      applicationCategory: "DeveloperApplication",
      operatingSystem: "Windows, macOS, Linux",
      softwareVersion: PUBLISHED_VERSION,
      url: "https://pytxo.com",
      downloadUrl: NPM_URL,
      image: `https://pytxo.com${SHARE_IMAGE.url}`,
      description: PRODUCT_SUMMARY,
    },
  ],
} as const;

const FAQ = [
  { q: "Which agents does it work with?", a: "Any agent that leaves its work on a branch. With no arguments Pytxo checks branches named claude/*, codex/*, copilot/*, cursor/*, devin/*, jules/* and agent/*, plus every branch in a linked worktree, that have a commit in the last 14 days. Name other branches yourself, or set git config pytxo.branches." },
  { q: "How does it find my tests?", a: "It reads the target branch and picks the usual command for package.json (npm, pnpm, yarn or bun, installing dependencies first when there are any), Cargo, Go, uv, Poetry, pytest, Deno, Gradle, Maven, .NET, Mix, RSpec, Rake, Composer or a Makefile test target. Or pass your own with -c." },
  { q: "Does it change my repository?", a: "No. The merge and the tests happen in a throwaway worktree. Only --apply lands a ready result on the target branch, and it refuses if that branch moved during the check." },
  { q: "What does a pass prove?", a: "That your tests pass on the merge of the branches. Pytxo catches what your tests catch: a break they don't cover can still land. Merge conflicts are blocked, not resolved." },
  { q: "What if a test is flaky?", a: "When the combined change fails, Pytxo runs each branch alone and, if every branch passes alone, the combined change once more. A failure that does not repeat is reported as flaky, not as a break. A combined run that times out is not run again, and the branches do not run alone after it." },
  { q: "Does it send my code anywhere?", a: "No. The check runs on your machine or in your own CI. It calls no model, needs no account, and sends nothing anywhere." },
  { q: "Is the GitHub Action safe on public repositories?", a: "It runs on pull_request only and refuses pull_request_target. A fork's pull request never joins another pull request's run: it is checked in its own run, with GitHub's read-only token and no secrets." },
  { q: "What does it cost?", a: "Pytxo is free on your machine and on public repositories, and open source under MIT. A paid plan for private repositories in CI is planned; until it opens, the Action is free everywhere." },
  { q: "What happened to Pytxo Desktop?", a: "It is frozen and still a free download for Windows. Pytxo now does one thing: the check." },
] as const;

export default function HomePage() {
  return (
    <>
      <script
        type="application/ld+json"
        // sloplint-disable-next-line SEC-007 -- constant structured data; "<" is escaped below
        dangerouslySetInnerHTML={{
          __html: JSON.stringify(STRUCTURED_DATA).replace(/</g, "\\u003c"),
        }}
      />
      <Hero />
      <HowItWorks />
      <CiSection />
      <CompareSection />
      <ProofSection />
      <CompatibilitySection />
      <section
        className="mx-auto grid max-w-[92rem] gap-12 border-t border-white/10 px-4 py-[clamp(4.5rem,7vw,7.5rem)] sm:px-6 lg:grid-cols-[0.72fr_1.28fr] lg:px-10"
        aria-labelledby="faq-title"
        data-testid="faq-section"
      >
        <div>
          <h2
            id="faq-title"
            className="max-w-md text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]"
          >
            Questions
          </h2>
          <p className="mt-6 max-w-sm text-sm leading-relaxed text-[#8d8d96]">
            Your agents write the branches. Your tests decide whether they go together.
          </p>
        </div>
        <Accordion type="single" collapsible className="w-full border-t border-white/10">
          {FAQ.map((item) => (
            <AccordionItem key={item.q} value={item.q} className="border-white/10 py-1">
              <AccordionTrigger className="py-5 text-left text-base hover:no-underline sm:text-lg">
                {item.q}
              </AccordionTrigger>
              <AccordionContent className="max-w-3xl pb-6 text-sm leading-relaxed text-[#a9a9b2] sm:text-base">
                {item.a}
              </AccordionContent>
            </AccordionItem>
          ))}
        </Accordion>
      </section>
      <GetItSection />
    </>
  );
}
