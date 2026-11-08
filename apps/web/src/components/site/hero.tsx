import { CopyCommand } from "@/components/site/copy-command";
import { MarkField } from "@/components/site/mark-field";
import s from "@/components/site/site.module.css";
import { SOURCE_URL } from "@/lib/site";
import { cn } from "@/lib/utils";

// data-keep tells the mark what to keep readable: measure the text or the box, and how far to dim the glyphs under it.
export function Hero() {
  return (
    <section className={cn(s.stage, s.hero)} id="top" aria-labelledby="hero-title" data-testid="marketing-hero">
      <MarkField />
      <div className={s.wrap}>
        <h1 id="hero-title">
          <span data-keep="text:0.22">Clear your agent</span>{" "}
          <span data-keep="text:0.22">PR pile in one command.</span>
        </h1>
        <p className={s.pitch} data-keep="text:0.1">
          Pytxo tests your agents&apos; branches together, lands the ones that pass, and gives each of the rest the
          reason and a fix prompt for its agent. Your own tests decide, not an AI model.
        </p>
        <div className={s.act}>
          <CopyCommand keep="box:0.1" />
          <a className={s.quiet} href={SOURCE_URL} data-keep="box:0.1">Source on GitHub</a>
        </div>
      </div>
    </section>
  );
}
