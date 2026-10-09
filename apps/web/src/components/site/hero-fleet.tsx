"use client";

import Image from "next/image";
import { useEffect, useRef, useState } from "react";

/**
 * The hero capture, then the same Desktop view playing a sample run once it
 * scrolls into view. The still stays underneath as the poster and the only
 * thing shown when the visitor prefers reduced motion.
 */
export function HeroFleet({ alt }: { alt: string }) {
  const video = useRef<HTMLVideoElement>(null);
  const [playing, setPlaying] = useState(false);

  useEffect(() => {
    const element = video.current;
    if (!element || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    let ended = false;
    const observer = new IntersectionObserver(([entry]) => {
      if (entry.isIntersecting) {
        if (ended) { element.currentTime = 0; ended = false; }
        void element.play().catch(() => {});
      } else element.pause();
    }, { threshold: .35 });
    const onEnd = () => { ended = true; };
    element.addEventListener("ended", onEnd);
    observer.observe(element);
    return () => { observer.disconnect(); element.removeEventListener("ended", onEnd); };
  }, []);

  return <div className="relative">
    <Image src="/product/fleet-1600x1000.png" alt={alt} width={1600} height={1000} sizes="(min-width: 1280px) 78rem, 100vw" priority className="h-auto w-full" />
    <video
      ref={video}
      className={`absolute inset-0 h-full w-full transition-opacity duration-500 ${playing ? "opacity-100" : "opacity-0"}`}
      src="/media/fleet-run-1600x1000.mp4"
      muted
      playsInline
      preload="none"
      aria-hidden="true"
      onPlaying={() => setPlaying(true)}
    />
  </div>;
}
