import Image from "next/image";

/**
 * One capture, large enough to read. The annotations are ordinary text beneath
 * the image rather than overlaid callouts, so nothing occludes the product and
 * the list stays legible on a phone.
 */
const ANNOTATIONS = [
  {
    n: "01",
    title: "Run ledger",
    body: "Every agent task grouped by the wave that scheduled it, with the state the orchestrator actually reported and the paths the task claimed.",
  },
  {
    n: "02",
    title: "Reported, not predicted",
    body: "A task is Completed, Running, Queued, or Failed with its exit code. There is no progress bar, because the orchestrator does not report per-agent progress.",
  },
  {
    n: "03",
    title: "Commit boundary",
    body: "The panel that shows the prepared package, effective permission profile, and journaled Apply state. It stays visible so repository impact is never a surprise.",
  },
  {
    n: "04",
    title: "Enforcement receipt",
    body: "Four isolation surfaces, each recorded as enforced, advisory, unavailable, or bypassed. Advisory is not a pass.",
  },
] as const;

export function ProductSection() {
  return (
    <section
      className="section-pad mx-auto max-w-[92rem] lg:px-10"
      aria-labelledby="product-title"
      data-testid="product-section"
    >
      <div className="max-w-[46rem]">
        <h2
          id="product-title"
          className="text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]"
        >
          One screen while the work runs.
        </h2>
        <p className="mt-7 text-base leading-relaxed text-[#a9a9b2] sm:text-lg">
          The left side is what happened. The right side is what you are about to
          authorise. Nothing else competes for the space.
        </p>
      </div>

      <figure className="mt-14">
        <div className="overflow-hidden rounded-[8px] border border-white/15 bg-[#09090b] shadow-[0_40px_120px_rgba(0,0,0,0.5)]">
          <Image
            src="/product/work-1600x1000.png"
            alt="Pytxo Desktop Work: agent tasks grouped by wave on the left, the commit boundary panel with a permission enforcement receipt on the right"
            width={1600}
            height={1000}
            className="h-auto w-full"
            sizes="(max-width: 1023px) 100vw, 92vw"
          />
        </div>
        <figcaption className="mt-4 font-mono text-[12px] text-[#6f6f79]">
          Pytxo Desktop, Work. 1600&times;1000 browser capture with preview fixtures; this screen illustrates the interface, not a live mission result.
        </figcaption>
      </figure>

      <ol className="mt-14 grid gap-x-12 gap-y-10 sm:grid-cols-2 lg:grid-cols-4">
        {ANNOTATIONS.map((item) => (
          <li key={item.n} className="border-t border-[var(--aperture-line)] pt-5">
            <span className="font-mono text-[12px] text-[#6f6f79]">{item.n}</span>
            <h3 className="mt-3 text-lg tracking-[-0.02em]">{item.title}</h3>
            <p className="mt-3 text-sm leading-relaxed text-[#8d8d96]">{item.body}</p>
          </li>
        ))}
      </ol>
    </section>
  );
}
