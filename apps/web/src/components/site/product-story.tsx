"use client";

import { useRef } from "react";
import Image from "next/image";
import { useGSAP } from "@gsap/react";
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(useGSAP, ScrollTrigger);

const STORY = [
  {
    title: "Plan before anything runs",
    description:
      "Turn one outcome into editable tasks, execution waves, path claims, permissions, and agent assignments.",
    image: "/product/flow-1600x1000.png",
    alt: "Pytxo Desktop New mission showing an editable outcome and a reviewable execution plan",
  },
  {
    title: "Run in isolation",
    description:
      "Supervise active agents, path locks, approval demand, sandbox coverage, and estimated spend from one local control plane.",
    image: "/product/operations-1600x1000.png",
    alt: "Pytxo Desktop Ops showing Needs you approvals and Running mission rows",
  },
  {
    title: "Review and approve",
    description:
      "Inspect the requester, workspace, action, and run evidence before isolated changes reach the repository.",
    image: "/product/approvals-1600x1000.png",
    alt: "Pytxo Approvals showing a selected Blast Shield flush with requester and run evidence",
  },
] as const;

export function ProductStory() {
  const root = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const media = gsap.matchMedia();

      media.add(
        "(min-width: 1024px) and (prefers-reduced-motion: no-preference)",
        () => {
          const section = root.current;
          const intro = section?.querySelector<HTMLElement>("[data-story-intro]");
          const track = section?.querySelector<HTMLElement>("[data-story-track]");
          const cards = gsap.utils.toArray<HTMLElement>("[data-story-card]", section);
          if (!section || !intro || !track || !cards.length) return;

          ScrollTrigger.create({
            trigger: track,
            start: "top top+=96",
            end: "bottom bottom-=96",
            pin: intro,
            pinSpacing: false,
            anticipatePin: 1,
          });

          for (const card of cards) {
            gsap.fromTo(
              card,
              { opacity: 0.55, scale: 0.96 },
              {
                opacity: 1,
                scale: 1,
                ease: "none",
                scrollTrigger: {
                  trigger: card,
                  start: "top 82%",
                  end: "center 52%",
                  scrub: 0.6,
                },
              },
            );

            gsap.to(card, {
              opacity: 0.42,
              scale: 0.98,
              ease: "none",
              scrollTrigger: {
                trigger: card,
                start: "center 28%",
                end: "bottom 8%",
                scrub: 0.6,
              },
            });
          }

          ScrollTrigger.refresh();
        },
      );

      return () => media.revert();
    },
    { scope: root },
  );

  return (
    <section
      ref={root}
      className="border-y border-border bg-card/10"
      aria-labelledby="product-story-title"
      data-testid="product-story"
    >
      <div className="section-pad mx-auto grid max-w-7xl gap-10 lg:grid-cols-[minmax(0,0.62fr)_minmax(0,1.38fr)] lg:gap-16">
        <div data-story-intro className="h-fit max-w-lg">
          <h2 id="product-story-title" className="text-3xl sm:text-4xl">
            From mission to verified result
          </h2>
          <p className="mt-4 max-w-md leading-relaxed text-muted-foreground">
            Pytxo makes the operating sequence inspectable: plan the work, run it in isolation,
            then decide what reaches your repository.
          </p>
        </div>

        <div data-story-track className="grid gap-8 lg:gap-12">
          {STORY.map((item) => (
            <article
              key={item.title}
              data-story-card
              className="overflow-hidden rounded-[var(--radius-lg)] border border-border bg-background shadow-[0_24px_70px_-52px_rgba(45,212,191,0.48)]"
            >
              <div className="px-5 py-5 sm:px-7 sm:py-6">
                <h3 className="text-xl font-semibold tracking-tight">{item.title}</h3>
                <p className="mt-2 max-w-2xl text-sm leading-relaxed text-muted-foreground sm:text-base">
                  {item.description}
                </p>
              </div>
              <Image
                src={item.image}
                alt={item.alt}
                width={1600}
                height={1000}
                className="h-auto w-full border-t border-border"
                sizes="(max-width: 1023px) 100vw, 62vw"
              />
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
