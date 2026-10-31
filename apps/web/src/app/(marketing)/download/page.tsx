import Link from "next/link";

import { DownloadDesktop } from "@/components/site/download-desktop";
import { InstallSnippet } from "@/components/site/install-snippet";
import s from "@/components/site/site.module.css";
import { pageMetadata } from "@/lib/page-metadata";
import {
  GITHUB_URL,
  INSTALL_PS1_CMD,
  INSTALL_SH_CMD,
  NPM_INSTALL,
  NPM_URL,
  PUBLISHED_VERSION,
  RELEASES_URL,
  SOURCE_URL,
} from "@/lib/site";
import { cn } from "@/lib/utils";

export const metadata = pageMetadata(
  "/download",
  "Download · Pytxo",
  "The Pytxo CLI for macOS, Linux and Windows, and Pytxo Desktop for Windows, which is frozen.",
);

const NEXT_STEPS = [
  { href: "/docs/getting-started/quickstart", label: "Quickstart" },
  { href: "/docs/getting-started/land-in-ci", label: "GitHub Action" },
  { href: "/docs/getting-started/agent-hooks", label: "Claude Code and Codex" },
  { href: "/docs/reference/cli", label: "CLI reference" },
  { href: "/docs/getting-started/install", label: "Install by platform" },
  { href: "/docs/concepts/desktop", label: "Pytxo Desktop overview" },
];

export default function DownloadPage() {
  return (
    <div className={s.page}>
      <div className={cn(s.wrap, s.inner)}>
        <div className={s.head}>
          <h1>Download</h1>
          <span className={s.ver}>v{PUBLISHED_VERSION}</span>
        </div>
        <p className={s.lede}>
          The check needs no download: run <code>npx pytxo demo</code>, then <code>npx pytxo</code> in your repository.
          Install the CLI below to keep it on your PATH. Pytxo Desktop is frozen: no new features, and still a free
          download for Windows.
        </p>

        <section className={s.part}>
          <div className={s.head}>
            <h2>CLI</h2>
            <span className={s.dim}>The check, for macOS, Linux and Windows</span>
          </div>
          <p className={s.note}>
            Checks your agents&apos; branches together. <code>npx pytxo</code> runs it without installing; these keep{" "}
            <code>pytxo</code> on your PATH.
          </p>
          <InstallSnippet
            label={<><a href={NPM_URL} target="_blank" rel="noopener noreferrer">npm</a> · Node 18+</>}
            copyLabel="Copy npm setup commands"
          >{`${NPM_INSTALL}\npytxo demo`}</InstallSnippet>

          <h3>Install script</h3>
          <p className={s.note}>
            Windows PowerShell is first-class. macOS and Linux scripts are also available. No Node required.
          </p>
          <InstallSnippet label="Windows PowerShell" copyLabel="Copy Windows PowerShell install command">{INSTALL_PS1_CMD}</InstallSnippet>
          <InstallSnippet label="macOS / Linux" copyLabel="Copy macOS and Linux install command">{INSTALL_SH_CMD}</InstallSnippet>
          <p className={s.note}>
            Binaries and scripts live in the public{" "}
            <a href={GITHUB_URL} target="_blank" rel="noopener noreferrer">pytxo-releases</a> repository. The
            MIT-licensed source is at{" "}
            <a href={SOURCE_URL} target="_blank" rel="noopener noreferrer">Pytxo-dev/pytxo</a>.
          </p>
          <ul className={s.rows}>
            <li>
              <span>Prebuilt binaries <span className={s.dim}>Manual download per platform from GitHub Releases.</span></span>
              <a href={RELEASES_URL} target="_blank" rel="noopener noreferrer" className="touch-target relative">Browse releases</a>
            </li>
          </ul>
        </section>

        <DownloadDesktop />

        <section className={s.part} data-testid="next-steps">
          <h2>Next steps</h2>
          <ul className={cn(s.rows, s.grid2)}>
            {NEXT_STEPS.map((item) => (
              <li key={item.href}>
                <Link href={item.href} className="touch-target relative">{item.label}</Link>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </div>
  );
}
