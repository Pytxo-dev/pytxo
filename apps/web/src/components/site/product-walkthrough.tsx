"use client";

import Image from "next/image";
import { animate } from "motion/mini";
import { useEffect, useRef, useState } from "react";
import { CANDIDATE_VERSION } from "@/lib/site";

// Kokonut Smooth Tab's stable-frame pattern, implemented locally with complete
// keyboard/tabpanel semantics. These are labeled preview captures, not a demo run.
const VIEWS = [
  {
    id: "work", title: "Execution", image: "work",
    description: "Follow recorded workers on the canvas, inspect their evidence, and open output when needed. Review stays within reach.",
    alt: "Pytxo Work cockpit preview with recorded worker dependencies and the Review changes action",
    details: [
      { label: "A run needs a decision", alt: "Work header shows a decision is needed and offers Review changes", centerX: 1390, top: 90, height: 190 },
      { label: "A worker's dependency is visible", alt: "The UI worker is awaiting a result and requires the plan task", centerX: 900, top: 545, height: 145 },
    ],
  },
  {
    id: "review", title: "Review & Apply", image: "run-review",
    description: "Inspect the exact prepared files and recorded checks. Apply requires a verified, non-empty, fresh candidate.",
    alt: "Pytxo Review preview with exact prepared file contents and review actions",
    details: [
      { label: "Checks belong to this candidate", alt: "Review shows combined checks passed and the recorded verification result", centerX: 1090, top: 185, height: 190 },
      { label: "Apply is an explicit decision", alt: "Review offers Apply reviewed changes at the repository boundary", centerX: 1350, top: 895, height: 100 },
    ],
  },
  {
    id: "history", title: "Recorded outcome", image: "history",
    description: "Return to the recorded run and its evidence. A saved history entry is not itself proof of a successful Apply.",
    alt: "Pytxo History preview with recorded runs and their states",
    details: [
      { label: "Execution and Apply are separate", alt: "History records Running and No confirmed Apply as separate states", centerX: 675, top: 200, height: 155 },
      { label: "Missing checks stay visible", alt: "History shows checks not recorded for the prepared candidate", centerX: 1200, top: 455, height: 90 },
    ],
  },
] as const;

export function ProductWalkthrough() {
  const [active, setActive] = useState(1);
  const indicator = useRef<HTMLSpanElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const buttons = useRef<Array<HTMLButtonElement | null>>([]);
  const initialized = useRef(false);
  useEffect(() => {
    const marker = indicator.current;
    const content = panel.current;
    if (!marker || !content) return;
    const preference = matchMedia("(prefers-reduced-motion: reduce)");
    const target = `translateX(${active * 100}%)`;
    let movement: ReturnType<typeof animate> | undefined;
    let reveal: ReturnType<typeof animate> | undefined;
    const settle = () => {
      movement?.cancel(); reveal?.cancel();
      marker.style.transform = target;
      content.style.opacity = "1";
    };
    if (!initialized.current || preference.matches) settle();
    else {
      movement = animate(marker, { transform: target }, { duration: 0.18, ease: [0.2, 0, 0, 1] });
      reveal = animate(content, { opacity: [0.8, 1] }, { duration: 0.18 });
    }
    initialized.current = true;
    const preferenceChanged = () => { if (preference.matches) settle(); };
    preference.addEventListener("change", preferenceChanged);
    return () => {
      // Retain the current marker position when the next selection interrupts it.
      movement?.stop(); reveal?.cancel();
      preference.removeEventListener("change", preferenceChanged);
    };
  }, [active]);

  function select(index: number) {
    setActive(index);
    buttons.current[index]?.focus();
  }

  return (
    <figure className="mt-10 lg:mt-14" data-testid="product-walkthrough">
      <div className="overflow-hidden rounded-[6px] border border-white/15 bg-[#09090b]">
        <div className="execution-trace" aria-hidden />
        <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 border-b border-white/15 px-4 py-3 text-xs sm:px-6">
          <span className="font-mono uppercase tracking-[0.04em] text-[#c4c4cc]">Pytxo Desktop interface preview</span>
          <span className="font-medium text-[#f5f5f7]">Unpublished v{CANDIDATE_VERSION} · browser fixture</span>
        </div>
        <div className="relative border-b border-white/15">
          <div role="tablist" aria-label="Explore Pytxo Desktop" className="grid grid-cols-3">
            {VIEWS.map((view, index) => (
              <button
                key={view.id}
                id={`product-tab-${view.id}`}
                role="tab"
                aria-selected={active === index}
                aria-controls={`product-panel-${view.id}`}
                tabIndex={active === index ? 0 : -1}
                ref={node => { buttons.current[index] = node; }}
                onClick={() => setActive(index)}
                onKeyDown={event => {
                  const target = event.key === "ArrowRight" ? (index + 1) % VIEWS.length
                    : event.key === "ArrowLeft" ? (index + VIEWS.length - 1) % VIEWS.length
                    : event.key === "Home" ? 0 : event.key === "End" ? VIEWS.length - 1 : null;
                  if (target !== null) { event.preventDefault(); select(target); }
                }}
                className="relative min-h-12 px-2 py-3 text-xs text-[#a9a9b2] aria-selected:bg-white/[0.04] aria-selected:text-white hover:text-white focus-visible:z-10 focus-visible:outline-2 focus-visible:-outline-offset-4 focus-visible:outline-[#a59bff] sm:text-sm"
              >{view.title}</button>
            ))}
          </div>
          <span ref={indicator} aria-hidden className="pointer-events-none absolute bottom-0 left-0 h-0.5 w-1/3 bg-[#a59bff]" />
        </div>
        {VIEWS.map((view, index) => (
          <div
            key={view.id}
            id={`product-panel-${view.id}`}
            role="tabpanel"
            aria-labelledby={`product-tab-${view.id}`}
            hidden={active !== index}
            tabIndex={0}
            ref={active === index ? panel : undefined}
            className="focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-[#a59bff]"
          >
            <p className="min-h-24 border-b border-white/10 px-4 py-4 text-sm leading-relaxed text-[#a9a9b2] sm:min-h-24 sm:px-6 lg:min-h-16">{view.description}</p>
            <div className="grid md:grid-cols-2 xl:hidden" data-testid="mobile-product-details">
              {view.details.map((detail) => (
                <div key={detail.label} className="min-w-0 border-b border-white/10 p-4 last:border-b-0 md:border-b-0 md:even:border-l">
                  <p className="mb-3 text-sm font-medium text-[#f5f5f7]">{detail.label}</p>
                  <div className="relative overflow-hidden rounded-[4px] bg-[#09090b]" style={{ height: detail.height }}>
                    <Image
                      src={`/product/${view.image}-1600x1000.png`}
                      alt={detail.alt}
                      width={1600}
                      height={1000}
                      className="absolute h-[1000px] w-[1600px] max-w-none"
                      style={{ left: `calc(50% - ${detail.centerX}px)`, top: -detail.top }}
                      sizes="1600px"
                    />
                  </div>
                </div>
              ))}
            </div>
            <Image
              src={`/product/${view.image}-1600x1000.png`}
              alt={view.alt}
              width={1600}
              height={1000}
              className="hidden h-auto w-full xl:block"
              sizes="(min-width: 1280px) 92vw, 1600px"
            />
            <a href={`/product/${view.image}-1600x1000.png`} className="block border-t border-white/10 px-4 py-3 text-xs text-[#a9a9b2] underline underline-offset-4 hover:text-white focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-[#a59bff] sm:px-6">Open full-size {view.id} capture</a>
          </div>
        ))}
      </div>
      <figcaption className="mt-4 text-xs leading-relaxed text-[#8d8d96]">
        Unpublished {CANDIDATE_VERSION} candidate. Interface previews using browser fixtures, not a recorded mission or proof of execution.
      </figcaption>
    </figure>
  );
}
