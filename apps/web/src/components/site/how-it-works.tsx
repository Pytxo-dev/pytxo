import Image from "next/image";

const STEPS = [
  {
    title: "Describe it once. Get a plan.",
    body: "Pytxo turns your request into tasks, gives each task the files it owns, and orders anything that shares a file. Choose which agents take part and edit any step before it runs.",
    image: "flow",
    alt: "Pytxo New work screen with a request on the left and the reviewed plan on the right: three steps, each with its task and assigned agent",
  },
  {
    title: "Agents work side by side.",
    body: "Each task runs in its own isolated copy of your project with the CLI you assigned. When a task finishes, Pytxo runs your checks on it. An agent saying “done” is not a pass.",
    image: "work",
    alt: "Pytxo Work view with the run's tasks laid out by step: a finished plan task, an active worker and a queued test task that waits for it",
  },
  {
    title: "Review one change. Apply exactly that.",
    body: "See every changed line, which agent wrote it, and the checks Pytxo ran on all the changes together. Apply writes those exact files and refuses if your project changed after you reviewed.",
    image: "run-review",
    alt: "Pytxo Review screen showing changed files, a line-by-line diff, checks passed, and the Apply reviewed changes action",
  },
] as const;

export function HowItWorks() {
  return <section id="how" className="scroll-mt-20 border-b border-white/10" aria-labelledby="how-title" data-testid="how-it-works">
    <div className="mx-auto max-w-[92rem] px-4 py-[clamp(4.5rem,7vw,7.5rem)] sm:px-6 lg:px-10">
      <h2 id="how-title" className="max-w-[22ch] text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">How it works</h2>
      <ol className="mt-14 grid gap-16 lg:gap-24">
        {STEPS.map((step, index) => <li key={step.image} className={`grid items-center gap-8 lg:gap-14 ${index % 2 ? "lg:grid-cols-[minmax(0,1.6fr)_minmax(0,0.8fr)]" : "lg:grid-cols-[minmax(0,0.8fr)_minmax(0,1.6fr)]"}`}>
          <div className={index % 2 ? "lg:order-2" : undefined}>
            <span className="font-mono text-sm text-[#8d8d96]">0{index + 1}</span>
            <h3 className="mt-3 text-2xl leading-tight tracking-[-0.03em] sm:text-3xl">{step.title}</h3>
            <p className="mt-5 max-w-[44ch] text-base leading-relaxed text-[#b4b4bd]">{step.body}</p>
          </div>
          <div className="overflow-hidden rounded-lg border border-white/10 bg-[#09090b]">
            <Image src={`/product/${step.image}-1600x1000.png`} alt={step.alt} width={1600} height={1000} sizes="(min-width: 1024px) 60vw, 100vw" className="h-auto w-full" />
          </div>
        </li>)}
      </ol>
      <p className="mt-12 text-sm text-[#8d8d96]">Screens are preview captures of the v1.2.2 Desktop with sample data. The <a className="aperture-link text-[#c4c4cc]" href="/evidence">evidence page</a> records real runs.</p>
    </div>
  </section>;
}
