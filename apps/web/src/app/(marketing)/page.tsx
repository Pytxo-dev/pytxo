import Link from "next/link";

import { CopyCommand } from "@/components/site/copy-command";
import { Hero } from "@/components/site/hero";
import { RecordedDemo } from "@/components/site/recorded-demo";
import s from "@/components/site/site.module.css";
import { pageMetadata } from "@/lib/page-metadata";
import { ACTION_USES, NPM_URL, PRODUCT_LINE, PRODUCT_SUMMARY, PUBLISHED_VERSION, SHARE_IMAGE } from "@/lib/site";
import { cn } from "@/lib/utils";
import demo from "../../../public/evidence/demo-landed-2.0.2-2026-11-12.json";

export const metadata = pageMetadata(
  "/",
  "Pytxo: land your coding agents' branches in one command",
  PRODUCT_SUMMARY,
);

const STRUCTURED_DATA = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "WebSite",
      "@id": "https://pytxo.com/#website",
      name: "Pytxo",
      url: "https://pytxo.com",
      description: PRODUCT_LINE,
    },
    {
      "@type": "SoftwareApplication",
      "@id": "https://pytxo.com/#software",
      name: "Pytxo",
      applicationCategory: "DeveloperApplication",
      operatingSystem: "Windows, macOS, Linux",
      softwareVersion: PUBLISHED_VERSION,
      url: "https://pytxo.com",
      downloadUrl: NPM_URL,
      image: `https://pytxo.com${SHARE_IMAGE.url}`,
      description: PRODUCT_SUMMARY,
    },
  ],
} as const;

const DEMO_URL = "/evidence/demo-landed-2.0.2-2026-11-12.json";
// What the recorded demo did with each branch, from its own "landed" and "left out" lines: the landed ones first, as
// the heading and the verdict read.
const OUTCOMES = demo.output
  .split("\n")
  .flatMap((line) => {
    const m = /^(landed|left out) +(\S+)/.exec(line);
    return m ? [{ branch: m[2], landed: m[1] === "landed" }] : [];
  })
  .sort((a, b) => Number(b.landed) - Number(a.landed));

const ACTION_AT = ACTION_USES.indexOf("actions/");

export default function HomePage() {
  return (
    <div className={s.page}>
      <script
        type="application/ld+json"
        // sloplint-disable-next-line SEC-007 -- constant structured data; "<" is escaped below
        dangerouslySetInnerHTML={{
          __html: JSON.stringify(STRUCTURED_DATA).replace(/</g, "\\u003c"),
        }}
      />
      <Hero />
      <RecordedDemo />

      <section className={s.sec} aria-labelledby="decide-title" data-testid="decide-section">
        <div className={s.wrap}>
          <h2 id="decide-title">What passes together lands.</h2>
          {/* The roles keep the table a table for screen readers where phones lay its rows out as a grid. */}
          <table className={s.ledger} role="table">
            <caption className="sr-only">What pytxo {demo.version} did with each of the demo&apos;s branches, from the recorded output</caption>
            <tbody role="rowgroup">
              {OUTCOMES.map((outcome) => (
                <tr key={outcome.branch} className={outcome.landed ? undefined : s.out} role="row">
                  <th scope="row" role="rowheader">{outcome.branch}</th>
                  <td className={s.res} role="cell"><span className={s.dot} aria-hidden="true" />{outcome.landed ? "landed" : "left out"}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <p className={s.verdict}><span>Two landed.</span> <span>Two left out, with the reason.</span></p>
          <div className={s.recFoot}>
            <a href={DEMO_URL}>Recorded output · pytxo&nbsp;{demo.version} · the&nbsp;same&nbsp;run</a>
          </div>
        </div>
      </section>

      <section className={s.sec} aria-labelledby="where-title" data-testid="where-section">
        <div className={s.wrap}>
          <h2 id="where-title">Where it runs.</h2>
          <ul className={s.where}>
            <li><p>In your terminal</p><code>npx pytxo --apply</code></li>
            <li>
              <p>When an agent stops, through the <Link href="/docs/getting-started/agent-hooks">Claude Code plugin or the Codex hook</Link></p>
              <code>npx pytxo hook</code>
            </li>
            <li>
              <p>On every agent pull request, with the <Link href="/docs/getting-started/land-in-ci">GitHub Action</Link></p>
              <code>uses: {ACTION_USES.slice(0, ACTION_AT)}<wbr />{ACTION_USES.slice(ACTION_AT)}</code>
            </li>
          </ul>
        </div>
      </section>

      <section className={cn(s.sec, s.close)} aria-labelledby="close-title" data-testid="close-section">
        <div className={s.wrap}>
          <h2 id="close-title"><span>Before the merge,</span> <span>not after.</span></h2>
          <div className={s.act}>
            <CopyCommand />
          </div>
        </div>
      </section>
    </div>
  );
}
