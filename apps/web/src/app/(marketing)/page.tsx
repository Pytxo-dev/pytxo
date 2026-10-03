import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { CompareSection } from "@/components/site/compare-section";
import { CompatibilitySection } from "@/components/site/compatibility-section";
import { GetItSection } from "@/components/site/get-it-section";
import { Hero } from "@/components/site/hero";
import { HowItWorks } from "@/components/site/how-it-works";
import { ProofSection } from "@/components/site/proof-section";
import { pageMetadata } from "@/lib/page-metadata";
import { PUBLISHED_VERSION } from "@/lib/site";

export const metadata = pageMetadata(
  "/",
  "Pytxo: many coding agents, one verified change",
  "Split one coding request across Codex, Claude Code, Cursor, OpenCode and Antigravity. Pytxo keeps them off each other's files, runs your checks on the combined result, and applies exactly what you reviewed.",
);

const STRUCTURED_DATA = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "WebSite",
      "@id": "https://pytxo.com/#website",
      name: "Pytxo",
      url: "https://pytxo.com",
      description: "The agent hypervisor for your repository: many coding agents, one verified change.",
    },
    {
      "@type": "SoftwareApplication",
      "@id": "https://pytxo.com/#software",
      name: "Pytxo",
      applicationCategory: "DeveloperApplication",
      operatingSystem: "Windows, macOS, Linux",
      softwareVersion: PUBLISHED_VERSION,
      url: "https://pytxo.com",
      downloadUrl: "https://pytxo.com/download",
      image: "https://pytxo.com/product/fleet-1600x1000.png",
      description:
        "Runs the coding-agent CLIs you already use on one planned job, checks the combined result, and applies exactly the reviewed repository changes.",
    },
  ],
} as const;

const FAQ = [
  { q: "Which agents does Pytxo run?", a: "The Desktop beta runs Codex, Claude Code, Cursor Agent, OpenCode and Antigravity. One installed and signed-in agent is enough to start; with more, Pytxo can give each task to a different one." },
  { q: "Do I need API keys or a Pytxo account?", a: "No Pytxo account for local work. Each agent uses its own sign-in, the same way it does in your terminal. Agents run with a clean environment, so API keys set in your shell are not passed to them." },
  { q: "How do agents avoid overwriting each other?", a: "Every task runs in its own isolated copy of your project, and the plan says which files each task owns. Tasks that share a file run one after another, so the later one starts from the earlier one's result." },
  { q: "What do the checks prove?", a: "Pytxo runs the commands you approve on each task and again on all the changes together. Passing means those commands passed on the exact files you are about to apply. It does not mean the code is right in every way." },
  { q: "What if my project changes after I review?", a: "Apply refuses and nothing is written. Refresh the review to recheck the changes against your current files, then Apply again." },
  { q: "What if Apply is interrupted?", a: "Pytxo journals every Apply. On restart it reconciles the attempt: confirmed as applied, rolled back, or flagged as needing recovery. It never reports a restore it could not confirm." },
  { q: "Does Pytxo sandbox everything an agent does?", a: "No. Pytxo controls what reaches your repository. It does not contain every file or network action an agent takes on your machine; each run's details list which protections were enforced and which were advisory." },
  { q: "Is it faster than one agent?", a: "Sometimes. Splitting helps when a job has parts that can run at once. A small fix is often quicker with one agent, and Pytxo works fine with one." },
] as const;

export default function HomePage() {
  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{
          __html: JSON.stringify(STRUCTURED_DATA).replace(/</g, "\\u003c"),
        }}
      />
      <Hero />
      <HowItWorks />
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
            Your agents write the code. You decide what lands.
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
