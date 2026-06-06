import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { ArchitectureSection } from "@/components/site/architecture-section";
import { CtaSection } from "@/components/site/cta-section";
import { FeatureGrid } from "@/components/site/feature-grid";
import { Hero } from "@/components/site/hero";
import { HowItWorks } from "@/components/site/how-it-works";
import { ProblemSection } from "@/components/site/problem-section";

const FAQ = [
  {
    q: "Is Pytxo an IDE or a cloud workspace?",
    a: "Neither. Pytxo is a local-first agent hypervisor: it runs shell-backed agents in PTYs, schedules waves, and logs structure. Your editor stays your editor.",
  },
  {
    q: "Which agents are supported?",
    a: "Any command you can run in a terminal — Claude Code, Codex, Antigravity CLI, Aider, or custom scripts via pytxo run --cmd.",
  },
  {
    q: "What is the Reality Deck?",
    a: "An optional Tauri desktop app for passive telemetry: runs, topology, and diffs — not a multi-pane terminal grid.",
  },
  {
    q: "How does Pytxo reduce cost?",
    a: "Signal Core scaffolds context instead of dumping whole files. Local execution avoids cloud ADE RAM/GPU overhead. You bring your own LLM keys (BYOK).",
  },
  {
    q: "Where are the docs?",
    a: "Consumer and developer guides live at pytxo.com/docs — install, tutorials, CLI reference, and architecture. The GitHub repo has additional internal design notes.",
  },
] as const;

export default function HomePage() {
  return (
    <>
      <Hero />
      <ProblemSection />
      <HowItWorks />
      <FeatureGrid />
      <ArchitectureSection />
      <CtaSection />
      <section className="mx-auto max-w-3xl px-4 pb-24 sm:px-6">
        <h2 className="mb-8 text-center text-2xl font-semibold">FAQ</h2>
        <Accordion type="single" collapsible className="glass-panel w-full rounded-xl px-4">
          {FAQ.map((item) => (
            <AccordionItem key={item.q} value={item.q} className="border-white/8">
              <AccordionTrigger className="text-left hover:no-underline">
                {item.q}
              </AccordionTrigger>
              <AccordionContent className="text-muted-foreground">
                {item.a}
              </AccordionContent>
            </AccordionItem>
          ))}
        </Accordion>
      </section>
    </>
  );
}
