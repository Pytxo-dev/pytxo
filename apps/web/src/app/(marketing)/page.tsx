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
import { MeasuredEvidence } from "@/components/site/measured-evidence";
import { ProductSection } from "@/components/site/product-section";
import { SituationSection } from "@/components/site/situation-section";

const FAQ = [
  {
    q: "Is Pytxo an IDE or a cloud workspace?",
    a: "Neither. Pytxo runs the terminal agents you already use, assigns path ownership and dependencies, records what changed, and prepares one result for review. Your editor stays your editor.",
  },
  {
    q: "Which agents are supported?",
    a: "Any command you can run in a terminal: Claude Code, Codex, Antigravity CLI (agy), Cursor Agent, OpenCode, Gemini CLI, Copilot CLI, Aider, or custom scripts via pytxo run --cmd. Editors connect through the local MCP hub instead.",
  },
  {
    q: "What is Pytxo Desktop?",
    a: "An optional native control surface for Work, History, and Setup, including approvals and applying eligible Orbit or Galaxy runs to one repository root. It reads the same local core the CLI uses, so it grants no extra authority.",
  },
  {
    q: "What does 'advisory only' mean on a receipt?",
    a: "Pytxo constrained the child process but the platform did not guarantee the constraint. It is recorded separately from enforced because it is weaker evidence, and it is never coloured as a pass.",
  },
  {
    q: "What happens if an Apply fails halfway?",
    a: "The run records the attempt, its outcome, and whether rollback was confirmed. If rollback was not confirmed, Pytxo says the working tree may be partially modified and offers the reconcile step rather than claiming a clean state.",
  },
  {
    q: "What are Workspaces and fleet runs?",
    a: "A Workspace coordinates work across folders. Fleet runs order steps across separate Git roots. Each repository root keeps its own Apply boundary; a fleet is not a cross-root transaction.",
  },
  {
    q: "How do I test with Antigravity (agy)?",
    a: "Install pytxo via npm, scaffold a test repo with tooling/test-envs, set cli_adapter = \"agy\" in pytxo.toml, and run pytxo run --cmd \"agy …\". See the docs guide for testing terminal agent CLIs.",
  },
  {
    q: "What are Signal Core, Blast Shield, and Race Shield?",
    a: "Signal Core starts reads with AST structure. Blast Shield isolates writes, stores the reviewed target blobs, and journals single-root Apply. Race Shield turns path ownership and dependencies into ordered waves.",
  },
  {
    q: "Where are the docs?",
    a: "Guides live at pytxo.com/docs: install, tutorials, CLI reference, Workspaces, fleet runs, and approval gates. ptyxo.com is unrelated.",
  },
] as const;

export default function HomePage() {
  return (
    <>
      <Hero />
      <SituationSection />
      <ProductSection />
      <BoundarySection />
      <CompatibilitySection />
      <MeasuredEvidence />
      <section
        className="section-pad mx-auto grid max-w-[92rem] gap-12 lg:grid-cols-[0.72fr_1.28fr] lg:px-10"
        aria-labelledby="faq-title"
      >
        <div>
          <h2
            id="faq-title"
            className="max-w-md text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]"
          >
            Questions with exact answers.
          </h2>
          <p className="mt-6 max-w-sm text-sm leading-relaxed text-[#85858f]">
            Pytxo is local-first orchestration, not another editor or opaque agent cloud.
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
