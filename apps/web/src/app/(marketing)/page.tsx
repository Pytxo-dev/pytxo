import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { ArchitectureSection } from "@/components/site/architecture-section";
import { AgentReadiness } from "@/components/site/agent-readiness";
import { CtaSection } from "@/components/site/cta-section";
import { DemoSection } from "@/components/site/demo-section";
import { FeatureGrid } from "@/components/site/feature-grid";
import { Hero } from "@/components/site/hero";
import { HowItWorks } from "@/components/site/how-it-works";
import { ProofBand } from "@/components/site/proof-band";
import { ScrollReveal } from "@/components/site/scroll-reveal";

const FAQ = [
  {
    q: "Is Pytxo an IDE or a cloud workspace?",
    a: "Pytxo runs the terminal agents you already use, assigns path ownership and dependencies, records what changed, and prepares one result for review. Your editor stays your editor.",
  },
  {
    q: "Which agents are supported?",
    a: "Any command you can run in a terminal: Claude Code, Codex, Antigravity CLI (agy), Aider, or custom scripts via pytxo run --cmd. Cursor integrates via MCP instead of --cmd.",
  },
  {
    q: "How do I test with Antigravity (agy)?",
    a: "Install pytxo via npm, scaffold a test repo with tooling/test-envs, set cli_adapter = \"agy\" in pytxo.toml, and run pytxo run --cmd \"agy …\". See the docs guide for testing terminal agent CLIs.",
  },
  {
    q: "What is Pytxo Desktop?",
    a: "An optional native UI for Ops, Missions, Approvals, and applying eligible Orbit or Galaxy runs to one repository root.",
  },
  {
    q: "What are Pytxo missions and Voice?",
    a: "A mission turns an outcome into a reviewable execution plan before anything runs. Voice transcribes speech locally into that same editable mission; Voice never approves actions or dispatches automatically.",
  },
  {
    q: "What are Workspaces and fleet runs?",
    a: "A Workspace coordinates work across folders. Fleet runs order steps across separate Git roots. Each repository root keeps its own Apply boundary; a fleet is not a cross-root transaction.",
  },
  {
    q: "How does Pytxo reduce cost?",
    a: "Signal Core measured an 82.9% scaffold-byte reduction across 185 tracked production files by sending syntax structure before full source. That figure is not a model-token, cost, or task-success claim. Local runs use your existing agent accounts and keys.",
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
      <DemoSection />
      <ProofBand />
      <HowItWorks />
      <ScrollReveal>
        <AgentReadiness />
      </ScrollReveal>
      <ScrollReveal delayMs={60}>
        <FeatureGrid />
      </ScrollReveal>
      <ScrollReveal delayMs={60}>
        <ArchitectureSection />
      </ScrollReveal>
      <ScrollReveal>
        <CtaSection />
      </ScrollReveal>
      <ScrollReveal delayMs={40}>
        <section className="mx-auto max-w-3xl px-4 pb-24 sm:px-6">
          <h2 className="mb-6 text-2xl font-semibold tracking-tight">FAQ</h2>
          <Accordion type="single" collapsible className="w-full border-t border-border">
            {FAQ.map((item) => (
              <AccordionItem key={item.q} value={item.q} className="border-border">
                <AccordionTrigger className="text-left hover:no-underline">
                  {item.q}
                </AccordionTrigger>
                <AccordionContent className="text-muted-foreground">{item.a}</AccordionContent>
              </AccordionItem>
            ))}
          </Accordion>
        </section>
      </ScrollReveal>
    </>
  );
}
