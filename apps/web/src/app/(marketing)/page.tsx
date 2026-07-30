import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { SupportedAgents } from "@/components/site/supported-agents";
import { ArchitectureSection } from "@/components/site/architecture-section";
import { AgentReadiness } from "@/components/site/agent-readiness";
import { CtaSection } from "@/components/site/cta-section";
import { DemoSection } from "@/components/site/demo-section";
import { FeatureGrid } from "@/components/site/feature-grid";
import { Hero } from "@/components/site/hero";
import { HowItWorks } from "@/components/site/how-it-works";
import { ProblemSection } from "@/components/site/problem-section";
import { ProofBand } from "@/components/site/proof-band";
import { ScrollReveal } from "@/components/site/scroll-reveal";

const FAQ = [
  {
    q: "Is Pytxo an IDE or a cloud workspace?",
    a: "Neither. Pytxo is a local agent hypervisor: it runs the tools you already use in the background, schedules their work, shows what changed, and lets you approve merges. Your editor stays your editor.",
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
    a: "An optional desktop app for planning missions with Flow, capturing a mission with local Voice, supervising runs, seeing affected code, approving changes, and reviewing diffs. Not a wall of terminals.",
  },
  {
    q: "What are Pytxo Flow and Voice?",
    a: "Flow turns a text mission into a reviewable execution plan before anything runs. Voice transcribes speech locally into that same editable Flow workflow; Voice never approves actions or dispatches automatically.",
  },
  {
    q: "What are Workspaces and fleet runs?",
    a: "A Workspace is one coordinated run across multiple folders (for example API + web). Fleet runs chain separate git repos when steps must finish in order.",
  },
  {
    q: "How does Pytxo reduce cost?",
    a: "Pytxo sends smaller, smarter context instead of whole files, runs locally without heavy cloud terminal UIs, and uses your own LLM API keys.",
  },
  {
    q: "What are Signal Core, Blast Shield, and Race Shield?",
    a: "Plain names for three built-in protections: smarter context (Signal), safe sandboxes until you approve (Blast), and no write collisions across agents (Race). Deep docs keep the product names.",
  },
  {
    q: "Where are the docs?",
    a: "Guides live at pytxo.com/docs: install, tutorials, CLI reference, Workspaces, fleet runs, and approval gates.",
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
      <ScrollReveal>
        <ProblemSection />
      </ScrollReveal>
      <ScrollReveal delayMs={60}>
        <FeatureGrid />
      </ScrollReveal>
      <ScrollReveal delayMs={40}>
        <SupportedAgents />
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
