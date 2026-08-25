"use client";

import { useSyncExternalStore } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DESKTOP_DOWNLOADS, DESKTOP_PRODUCT_NAME, PYTXO_VERSION } from "@/lib/site";
import { AppleIcon, detectOs, LinuxIcon, WindowsIcon, type DetectedOs } from "@/components/site/os-icons";
import { cn } from "@/lib/utils";

function subscribeOs() {
  return () => {};
}

function getOsSnapshot(): DetectedOs {
  return detectOs();
}

function getOsServerSnapshot(): DetectedOs {
  return "unknown";
}

const PLATFORMS = [
  {
    id: "windows" as const,
    label: "Windows",
    detail: ".msi · x64",
    href: DESKTOP_DOWNLOADS.windows,
    available: true,
    Icon: WindowsIcon,
    match: (os: DetectedOs) => os === "windows",
  },
  {
    id: "macos-arm" as const,
    label: "macOS",
    detail: "v1 build not available yet",
    href: DESKTOP_DOWNLOADS.macosArm,
    available: false,
    Icon: AppleIcon,
    match: (os: DetectedOs) => os === "macos",
  },
  {
    id: "macos-intel" as const,
    label: "macOS",
    detail: "v1 build not available yet",
    href: DESKTOP_DOWNLOADS.macosX64,
    available: false,
    Icon: AppleIcon,
    match: () => false,
  },
  {
    id: "linux" as const,
    label: "Linux",
    detail: "v1 build not available yet",
    href: DESKTOP_DOWNLOADS.linux,
    available: false,
    Icon: LinuxIcon,
    match: (os: DetectedOs) => os === "linux",
  },
] as const;

export function DownloadDesktop() {
  const os = useSyncExternalStore(subscribeOs, getOsSnapshot, getOsServerSnapshot);

  return (
    <section className="rounded-[4px] border border-border bg-card p-6 sm:p-8">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <div className="flex flex-wrap items-center gap-2">
            <h2 className="text-xl font-semibold tracking-tight sm:text-2xl">
              {DESKTOP_PRODUCT_NAME}
            </h2>
            <Badge
              variant="outline"
              className="border-border font-mono text-[10px] text-muted-foreground"
            >
              v{PYTXO_VERSION}
            </Badge>
          </div>
          <p className="mt-1 max-w-xl text-sm text-muted-foreground">
            Control surface for runs, structure, approvals, and diffs. Requires the CLI for
            orchestration.
          </p>
        </div>
        {os !== "unknown" ? (
          <Badge variant="outline" className="w-fit border-primary/30 bg-primary/10 text-primary">
            Suggested for your OS
          </Badge>
        ) : null}
      </div>

      <div className="mt-6 grid gap-3 sm:grid-cols-2">
        {PLATFORMS.map((platform) => {
          const suggested = platform.available && platform.match(os);
          const Icon = platform.Icon;
          const body = (
            <>
              <span
                className={cn(
                  "flex size-11 shrink-0 items-center justify-center rounded-[var(--radius-md)] border",
                  suggested
                    ? "border-primary/30 bg-primary/15 text-primary"
                    : "border-border bg-muted/30 text-foreground/80",
                )}
              >
                <Icon className="size-5" />
              </span>
              <span className="min-w-0 flex-1 text-left">
                <span className="flex items-center gap-2">
                  <span className="font-medium">{platform.label}</span>
                  {suggested ? (
                    <span className="text-[10px] font-medium uppercase tracking-wide text-primary">
                      Suggested
                    </span>
                  ) : null}
                </span>
                <span className="mt-0.5 block text-xs text-muted-foreground">
                  {platform.detail}
                </span>
              </span>
              <Button
                size="sm"
                variant={suggested ? "default" : "outline"}
                className={cn(
                  "shrink-0 pointer-events-none",
                  !suggested && "border-border",
                )}
                tabIndex={-1}
                disabled={!platform.available}
              >
                {platform.available ? "Download" : "Not yet"}
              </Button>
            </>
          );
          const className = cn(
            "group flex items-center gap-4 rounded-[var(--radius-md)] border px-4 py-4 transition-colors",
            suggested
              ? "border-primary/40 bg-primary/8 hover:bg-primary/12"
              : "border-border bg-background/40",
            platform.available ? "hover:border-border hover:bg-card/50" : "opacity-65",
          );

          return platform.available ? (
            <a key={platform.id} href={platform.href} className={className}>
              {body}
            </a>
          ) : (
            <div key={platform.id} className={className} aria-disabled="true">
              {body}
            </div>
          );
        })}
      </div>
    </section>
  );
}
