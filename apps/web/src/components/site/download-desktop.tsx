"use client";

import { useSyncExternalStore } from "react";

import s from "@/components/site/site.module.css";
import { DESKTOP_DOWNLOADS, DESKTOP_PRODUCT_NAME, PUBLISHED_VERSION } from "@/lib/site";

// Desktop is built for Windows only, so that is the one platform the page can suggest.
const subscribeOs = () => () => {};
const onWindows = () => /win/i.test(navigator.userAgent);
const onServer = () => false;

const PLATFORMS = [
  { id: "windows", label: "Windows", detail: ".msi · x64", href: DESKTOP_DOWNLOADS.windows, available: true },
  { id: "macos", label: "macOS", detail: "Desktop not built yet; use the CLI above", href: DESKTOP_DOWNLOADS.macosArm, available: false },
  { id: "linux", label: "Linux", detail: "Desktop not built yet; use the CLI above", href: DESKTOP_DOWNLOADS.linux, available: false },
] as const;

export function DownloadDesktop() {
  const windows = useSyncExternalStore(subscribeOs, onWindows, onServer);

  return (
    <section className={s.part}>
      <div className={s.head}>
        <h2>{DESKTOP_PRODUCT_NAME}</h2>
        <span className={s.ver}>v{PUBLISHED_VERSION}</span>
      </div>
      <p className={s.note}>
        Work, History, and Setup with the local orchestration core built in. Install a supported agent CLI separately;
        the Pytxo CLI is optional for Desktop.
      </p>
      <ul className={s.rows}>
        {PLATFORMS.map((platform) => (
          <li key={platform.id}>
            <span>
              {platform.label} <span className={s.dim}>{platform.detail}</span>
              {platform.available && windows ? <span className={s.dim}> · suggested for your OS</span> : null}
            </span>
            {platform.available ? (
              <a href={platform.href} className="touch-target relative">Download</a>
            ) : (
              <button type="button" disabled>Not yet</button>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}
