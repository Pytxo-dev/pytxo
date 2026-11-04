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
          <span data-keep="text:0.22">Land all your agents&apos;</span>{" "}
          <span data-keep="text:0.22">branches in one command.</span>
        </h1>
        <p className={s.pitch} data-keep="text:0.1">
          Pytxo merges them in a throwaway worktree, runs your tests and lands them only if they pass. When two break
          each other, nothing lands.
        </p>
        <div className={s.act}>
          <CopyCommand keep="box:0.1" />
          <a className={s.quiet} href={SOURCE_URL} data-keep="box:0.1">Source on GitHub</a>
        </div>
      </div>
    </section>
  );
}
