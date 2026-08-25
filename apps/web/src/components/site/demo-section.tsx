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
  const captionsUrl = process.env.NEXT_PUBLIC_DEMO_CAPTIONS_URL?.trim() ?? "";
  const media = embedUrl(raw);
  if (!media) return null;

  return (
    <section className="relative border-t border-border bg-background px-6 py-20 sm:py-24">
      <div className="mx-auto max-w-5xl">
        <ScrollReveal>
          <h2 className="max-w-2xl text-3xl font-semibold tracking-tight sm:text-4xl">
            Watch the reviewed Apply path.
          </h2>
          <p className="mt-3 max-w-xl text-sm leading-relaxed text-muted-foreground sm:text-base">
            For Orbit and Galaxy in one execution domain and one repository root,
            Run Review shows the prepared package and Apply uses only its stored bytes.
          </p>
        </ScrollReveal>

        <ScrollReveal className="mt-10">
          <div className="relative overflow-hidden rounded-[4px] border border-border bg-card">
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
                {captionsUrl ? (
                  <track
                    kind="captions"
                    src={captionsUrl}
                    srcLang="en"
                    label="English"
                  />
                ) : null}
              </video>
            )}
          </div>
        </ScrollReveal>
      </div>
    </section>
  );
}
