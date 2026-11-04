import Link from "next/link";

import { CopyCommand } from "@/components/site/copy-command";
import { Hero } from "@/components/site/hero";
import { RecordedDemo } from "@/components/site/recorded-demo";
import s from "@/components/site/site.module.css";
import { pageMetadata } from "@/lib/page-metadata";
import { ACTION_USES, NPM_URL, PRODUCT_LINE, PRODUCT_SUMMARY, PUBLISHED_VERSION, SHARE_IMAGE } from "@/lib/site";
import { cn } from "@/lib/utils";
import comment from "../../../public/evidence/comment-blocked-2.0.0-2026-10-30.json";

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

const COMMENT_URL = "/evidence/comment-blocked-2.0.0-2026-10-30.json";
// The checks table of the recorded pull request comment, in the order the check ran them: the combined change,
// each branch alone once that failed, and the combined change once more.
const RUNS = comment.output
  .split(/\r?\n/)
  .flatMap((row) => {
    const m = /^\| `[^`]+` \| `([^`]+)` \| \**(pass|fail)\** \| ([\d.]+)s \|$/.exec(row);
    if (!m) return [];
    const [, scope, result, seconds] = m;
    const alone = scope.startsWith("input:");
    return [{
      order: scope === "combined" ? 0 : alone ? 1 : 2,
      label: alone ? `${scope.slice("input:".length)} alone` : scope === "combined" ? "together" : "together, again",
      passed: result === "pass",
      seconds,
    }];
  })
  .sort((a, b) => a.order - b.order);

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
          <h2 id="decide-title">Together first. If that fails, each alone.</h2>
          {/* The roles keep the table a table for screen readers where phones lay its rows out as a grid. */}
          <table className={s.ledger} role="table">
            <caption className="sr-only">npm test runs from the pull request comment pytxo {comment.version} wrote for the demo&apos;s branches</caption>
            <tbody role="rowgroup">
              {RUNS.map((run) => (
                <tr key={run.label} className={run.passed ? s.pass : s.fail} role="row">
                  <th scope="row" role="rowheader">{run.label}</th>
                  <td className={s.res} role="cell"><span className={s.dot} aria-hidden="true" />{run.passed ? "pass" : "fail"}</td>
                  <td className={s.t} role="cell">{run.seconds}&nbsp;s</td>
                </tr>
              ))}
            </tbody>
          </table>
          <p className={s.verdict}>A break, not a flaky test. Blocked.</p>
          <div className={s.recFoot}>
            <a href={COMMENT_URL}>Pull request comment · pytxo&nbsp;{comment.version} · rendered locally</a>
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
