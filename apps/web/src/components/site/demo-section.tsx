"use client";

import { ScrollReveal } from "@/components/site/scroll-reveal";

function embedUrl(raw: string): { kind: "youtube" | "video"; src: string } | null {
  const url = raw.trim();
  if (!url) return null;
  try {
    const u = new URL(url);
    if (u.hostname.includes("youtube.com") || u.hostname === "youtu.be") {
      let id = "";
      if (u.hostname === "youtu.be") id = u.pathname.slice(1);
      else if (u.pathname.startsWith("/embed/")) id = u.pathname.split("/")[2] ?? "";
      else id = u.searchParams.get("v") ?? "";
      if (!id) return null;
      return { kind: "youtube", src: `https://www.youtube-nocookie.com/embed/${id}?rel=0` };
    }
  } catch {
    /* fall through — treat as direct media */
  }
  if (/\.(mp4|webm)(\?|$)/i.test(url) || url.startsWith("/")) {
    return { kind: "video", src: url };
  }
  return { kind: "video", src: url };
}

export function DemoSection() {
  const raw = process.env.NEXT_PUBLIC_DEMO_VIDEO_URL ?? "";
  const media = embedUrl(raw);
  if (!media) return null;

  return (
    <section className="relative border-t border-white/5 bg-[#020205] px-6 py-20 sm:py-24">
      <div className="mx-auto max-w-5xl">
        <ScrollReveal>
          <p className="font-mono text-[11px] uppercase tracking-[0.18em] text-teal-300/80">
            Demo
          </p>
          <h2 className="mt-3 max-w-2xl text-3xl font-semibold tracking-tight text-zinc-50 sm:text-4xl">
            See the control plane, not a wall of terminals.
          </h2>
          <p className="mt-3 max-w-xl text-sm leading-relaxed text-zinc-400 sm:text-base">
            Run agents you already use. Approve what lands. Keep keys in your environment.
          </p>
        </ScrollReveal>

        <ScrollReveal className="mt-10">
          <div className="relative overflow-hidden rounded-sm border border-white/10 bg-black shadow-[0_0_80px_-20px_rgba(45,212,191,0.25)]">
            {media.kind === "youtube" ? (
              <iframe
                title="Pytxo demo"
                src={media.src}
                className="aspect-video w-full"
                allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                allowFullScreen
                loading="lazy"
              />
            ) : (
              <video
                className="aspect-video w-full"
                controls
                playsInline
                preload="metadata"
                src={media.src}
              >
                <track kind="captions" />
              </video>
            )}
          </div>
        </ScrollReveal>
      </div>
    </section>
  );
}
