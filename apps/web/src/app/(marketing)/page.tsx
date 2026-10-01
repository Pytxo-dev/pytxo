import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { BoundarySection } from "@/components/site/boundary-section";
import { CompatibilitySection } from "@/components/site/compatibility-section";
import { GetItSection } from "@/components/site/get-it-section";
import { Hero } from "@/components/site/hero";
import { pageMetadata } from "@/lib/page-metadata";
import { PUBLISHED_VERSION } from "@/lib/site";

export const metadata = pageMetadata(
  "/",
  "Pytxo: Agent hypervisor for repository work",
  "Run coding agents in isolated copies of your repository. Verify the combined changes, review the exact diff, then explicitly Apply.",
);

const STRUCTURED_DATA = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "WebSite",
      "@id": "https://pytxo.com/#website",
      name: "Pytxo",
      url: "https://pytxo.com",
      description:
        "A local agent hypervisor for bounded coding work and reviewed repository changes.",
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
      image: "https://pytxo.com/product/work-1600x1000.png",
      description:
        "Coordinate coding-agent CLIs in isolated workspaces, inspect one prepared candidate, and explicitly Apply reviewed repository changes.",
    },
  ],
} as const;

const FAQ = [
 { q: "Does local work require a Pytxo account?", a: "No. Local Core can use your existing agent access without a Pytxo account. Your agent provider’s costs and authentication are separate." },
 { q: "What does verification prove?", a: "It records the configured commands run against the combined candidate. Passing checks are evidence for those checks, not a guarantee that all behavior is correct. Worker self-report is not verification." },
 { q: "What happens if Apply is interrupted?", a: "Pytxo records attempts and journal evidence. If a safe outcome cannot be established, recovery remains unresolved rather than claiming the repository was restored." },
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
      <BoundarySection />
      <CompatibilitySection />
      <section
        className="section-pad mx-auto grid max-w-[92rem] gap-12 lg:grid-cols-[0.72fr_1.28fr] lg:px-10"
        aria-labelledby="faq-title"
      >
        <div>
          <h2
            id="faq-title"
            className="max-w-md text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]"
          >
            Before you start.
          </h2>
          <p className="mt-6 max-w-sm text-sm leading-relaxed text-[#85858f]">
            Your agent does the coding. You stay in charge of the changes.
          </p>
        </div>
        <Accordion type="single" collapsible className="w-full border-t border-white/10">
          {FAQ.map((item) => (
            <AccordionItem key={item.q} value={item.q} className="border-white/10 py-1">
              <AccordionTrigger className="py-5 text-left text-base hover:no-underline sm:text-lg">
                {item.q}
              </AccordionTrigger>
              <AccordionContent className="max-w-3xl pb-6 text-sm leading-relaxed text-[#8d8d96] sm:text-base">
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
