---
title: Product review of pytxo.com and docs (v1.3.5)
slug: 2026-10-25-product-review
status: active
tags: [website, docs, review]
audience: [human, agent]
layer: presentation
created: 2026-10-25
updated: 2026-10-25
related: [[MOC-home]]
---

# Product review: pytxo.com, /docs, /download (2026-10-25)

Scope: every page in `sitemap.xml` (63 URLs), nav, footer and the 404 page on
https://pytxo.com, reviewed as eight personas against public release **1.3.5**.
Source: `apps/web` (Next.js site and the Fumadocs docs under
`apps/web/content/docs`). `apps/docs` is the retired Docusaurus package
(`apps/docs/README.md`) and is not deployed, so it was not reviewed.

## Counts

| Severity | Found | Fixed in source | Waiting on Matt |
|---|---|---|---|
| P0 | 1 | 1 | 0 |
| P1 | 10 | 6 | 4 |
| P2 | 14 | 6 | 8 |

Nothing is committed or deployed. Fixes are in the working tree of
`mbcz/product-review-a0207f`.

## Findings

Paths are relative to `apps/web` unless they start with `crates/` or `docs/`.

### P0

| # | URL | Persona | What happened | Evidence | Source | Status |
|---|---|---|---|---|---|---|
| 1 | /docs/compare/warp-oz | Skeptical buyer | The only source link for Warp Oz returns 404. Warp moved its agent docs to `/platform/`. | `curl -L https://docs.warp.dev/agent-platform` → 404; `https://docs.warp.dev/platform/` → 200 ("Cloud agents overview", lists the Oz web app) | `content/docs/compare/warp-oz.mdx:6` | Fixed |

### P1

| # | URL | Persona | What happened | Evidence | Source | Status |
|---|---|---|---|---|---|---|
| 2 | every marketing page | Mobile | Tapping a link in the mobile menu navigates but leaves the menu sheet open over the new page. | 375 px: open menu, tap Pybench → URL `/bench`, `[role=dialog]` count 1, page hidden behind the sheet | `src/components/site/header.tsx:89` | Fixed: links wrapped in `SheetClose` |
| 3 | any unknown URL | First-time visitor | 404 is the bare Next.js page: no header, footer or links, and it reuses the homepage title. | `/this-page-does-not-exist-xyz`: text "404 This page could not be found.", title "Pytxo: the landing layer for coding agents" | `src/app/not-found.tsx` (new) | Fixed: site header/footer, links to Docs, Download, Home; title "Page not found · Pytxo" |
| 4 | every marketing page | Accessibility | Footer bottom row (©, Terms, Privacy, Refunds, LOCAL BY DEFAULT) is `#66666f` 10 px on `#050507`: 3.58:1, below AA 4.5:1. | Lighthouse color-contrast on home, plans, download; own contrast sweep | `src/components/site/footer.tsx:107-115` | Fixed: `#7d7d87` (5.0:1, the footer's existing link colour) |
| 5 | /plans, /download, /terms, /privacy, /refunds | Accessibility | Links inside sentences are told apart by colour only (WCAG 1.4.1). | Lighthouse `link-in-text-block` on plans and download | `src/app/(marketing)/plans/page.tsx:123`, `src/app/(marketing)/download/page.tsx:99,108`, `src/components/site/legal-page.tsx:9`, `src/components/account-auth.tsx:318` | Fixed: underlined |
| 6 | every marketing page | Accessibility | No skip link to the main content. | No "skip" anchor in header or layouts | `src/components/site/header.tsx:35`, `src/app/(marketing)/layout.tsx:12` | Fixed: "Skip to content" shown on focus, `main#main` |
| 7 | /docs/troubleshooting | Docs follower | Troubleshooting says the guides describe v1.2.2 while the docs home and install page say 1.3.5. | "The installed app looks different — Check its version; these guides describe v1.2.2" | `content/docs/troubleshooting/index.mdx:20` | Fixed: points to the release on Download and to the version shown in Setup (verified in `apps/desktop/src/components/shell/UpdateControls.svelte:19`) |
| 8 | / (mobile) | Slow connection | Lighthouse mobile: performance 31, LCP 6.0 s, TBT 6.0 s, 1.48 MB. 571 KiB of it is 1600×1000 PNG screenshots served as-is, and every public page loads Clerk's UI bundle (184 KiB unused JS) for the header Sign in. Unthrottled runs were 1.8 s LCP, so this is a slow-phone problem, not a broken page. | `lh_home.json`: image-delivery-insight 571 KiB (`/product/flow-1600x1000.png`, `fleet-…`, `run-review-…`), unused-javascript Clerk | `next.config.ts:8` (`images.unoptimized: true`), `src/components/providers.tsx:15` (`ClerkProvider` around the whole site) | Proposal A, waits on Matt |
| 9 | /docs/* | Slow connection | Docs pages shift as web fonts load: CLS 0.179 on Install (above the 0.1 threshold). | Lighthouse layout-shifts: `article#nd-page ol > li`, caused by `sora-latin-400-normal.woff2`, `ibm-plex-mono-latin-400-normal.woff2` | `src/app/globals.css:7-10` (`@fontsource` imports, no fallback metrics) | Proposal B, waits on Matt |
| 10 | / | First-time visitor (macOS/Linux) | The hero's only primary button is "Download for Windows". A Mac visitor gets no primary action; the CLI path is the small "Land branches from the CLI ↓" link. | Desktop and mobile screenshots of the hero | `src/components/site/hero.tsx:28` | Proposal C, waits on Matt (hero CTA) |
| 11 | /plans | Skeptical buyer | "Free account · New in 1.3: Share a receipt as a public page" reads as shipped, but sharing has never been run end to end: `pytxo login` timed out twice and `receipts share` is carried as blocked in every checkpoint since 2026-10-16. | `CHECKPOINT.md:610`; on 1.3.5 `pytxo receipts share <id>` prints "this is an account perk: run `pytxo login` first" | `src/app/(marketing)/plans/page.tsx` (Free account card) | Waits on Matt: run `pytxo login` once at the browser and share one receipt, or mark the perk as coming soon |

### P2

| # | URL | Persona | What happened | Source | Status |
|---|---|---|---|---|---|
| 12 | /download | Buyer (macOS/Linux) | macOS and Linux cards said "v1 build not available yet" next to a v1.3.5 badge, and the "Suggested for your OS" badge showed on macOS/Linux with nothing suggested. | `src/components/site/download-desktop.tsx:36,45,76` | Fixed: "Desktop not built yet; use the CLI below"; badge only when the OS has a build |
| 13 | /bench, /docs/evidence/pybench | Search and share | Both pages are titled "Pybench · Pytxo". | `src/app/(marketing)/bench/page.tsx:10` | Fixed: "Pybench results · Pytxo" (matches the homepage link text) |
| 14 | /docs/evidence/pybench, first-use, signal-core | Developer | Links to evidence JSON files are client-side links, so Next prefetches them as pages and logs a 404 in the console on every view. | `src/components/mdx.tsx` | Fixed: file links under `/` render as plain anchors |
| 15 | /plans | Accessibility | Headings jump from h1 to h3 (plan cards). | `src/components/site/plan-roadmap-card.tsx:25` | Fixed: h2 |
| 16 | every marketing page | Accessibility | Header logo link is announced "Pytxo Pytxo" (image alt repeats the visible word). | `src/components/site/header.tsx:44` | Fixed: decorative `alt=""` |
| 17 | /docs/reference/cli | Developer | `pytxo project remove` exists in `--help` but not in the project subcommand table. | `content/docs/reference/cli.mdx:169` | Fixed |
| 18 | /plans | Skeptical buyer | Hero says Pro adds "unlimited history"; the Pro card lists "Unlimited shared receipts". Two names for one perk. | `src/app/(marketing)/plans/page.tsx:79` | Waits on Matt (pricing copy) |
| 19 | /docs/plans/ultra | Skeptical buyer | Docs nav lists "Ultra" under "Plans", but /plans has no Ultra tier, the page says managed inference is not end to end, and the documented `proxy_url = "https://link.pytxo.com"` returns 404. | `content/docs/plans/meta.json`, `content/docs/plans/ultra.mdx` | Waits on Matt: move it under Reference as a billing mode, or hide it until it ships |
| 20 | all pages | Search and share | All 65 pages share one OG image (`/product/work-1600x1000.png`), so every shared docs link looks the same. | `src/lib/page-metadata.ts:6`, `src/app/layout.tsx` | Proposal D |
| 21 | / | Search and share | Homepage meta description is 207 characters; search results cut at about 160. | `src/lib/site.ts` / `src/app/(marketing)/page.tsx` metadata | Waits on Matt (positioning copy) |
| 22 | /docs/reference/changelog | Docs follower | Opens with "This source candidate targets 1.3.5", which is internal release language. The sentence is pinned by the release verifier regex. | `content/docs/reference/changelog.mdx:6`, `tooling/scripts/verify-release-version.mjs:97` | Not fixed: change both together in a release PR |
| 23 | CLI | Developer | `pytxo --help` shows blank descriptions for `run`, `status`, `logs`, `stop`, and `agents` says "List ADE CLIs" (jargon). | `crates/pytxo-cli/src/main.rs:161,200` | Not fixed: needs a Rust build; small follow-up |
| 24 | /terms, /privacy, /refunds | Skeptical buyer | Legal contact is `contact@mbcz.xyz`, not a pytxo.com address. | `src/components/site/legal-page.tsx:16` | Waits on Matt |
| 25 | / | Skeptical buyer | The terminal and Pybench panels are labelled pytxo 1.3.0 while Download offers 1.3.5. Honest (recorded runs), but it reads as stale. | `src/components/site/land-section.tsx`, `src/components/site/proof-section.tsx`, `public/evidence/pybench-1.3.0-2026-10-10.json` | Note: re-run `pytxo bench` on the current release when convenient |

## Proposals (need approval; nothing implemented)

**A. Mobile weight on the homepage (P1 #8).** Two independent changes:
convert the `public/product/*-1600x1000.png` screenshots to WebP/AVIF at
sync time (`scripts/sync-product-assets.mjs`) with a 1x/2x `srcset`, which
keeps `images.unoptimized` and Vercel image billing as they are and saves
about 500 KiB; and load Clerk only on `/account`, `/sign-in`, `/sign-up` and
`/r/*`, rendering the header "Sign in" as a plain link elsewhere. Expected:
LCP on a throttled phone well under 4 s.

**B. Font shift on docs (P1 #9).** Move Sora and IBM Plex from `@fontsource`
CSS imports to `next/font/local` (same files), which adds size-adjusted
fallbacks and removes the shift. Typography looks the same once loaded.

**C. Hero call to action for macOS/Linux (P1 #10).** On macOS/Linux, swap the
hero's primary button to "Install the CLI" (scrolls to the land section with
the npm command) and keep "Download for Windows" on Windows. Same layout, the
label and target depend on the detected OS, as the Download page already does.

**D. Per-section share images (P2 #20).** One OG image each for Docs,
Download, Plans and Pybench, rendered from existing product captures.

## Verified, no finding

- **Docs follower, literally:** in a clean scratch prefix and a new Git repo
  with two agent branches, `npm i -g pytxo@1.3.5` (10.5 s, npm 11.18 prints the
  allow-scripts warning the docs describe), `pytxo --version` → `pytxo 1.3.5`,
  `pytxo land claude:… codex:… --check "node test.js" --per-input` → `READY`
  with "land it rerun with --apply", `--apply` → `LANDED` with one merge commit
  per branch, `pytxo receipts` lists both, `receipts show` prints
  `output_tail` and "digest matches". Every step of
  /docs/getting-started/first-landing worked as written.
- **Claims vs evidence:** homepage numbers (4/4 broken mains refused, 12/12,
  0 good changes refused, 3.1 s median, 7 tasks, 3 CLIs, 14/14 files, 22/22
  tests, 10 min 50 s) all match `public/evidence/pybench-1.3.0-2026-10-10.json`
  and `public/evidence/fleet-live-2026-10-10.json`.
- **Reference vs product:** every command in /docs/reference/cli exists in
  `pytxo --help` 1.3.5 and every documented flag appears in a subcommand's
  `--help`; all 33 keys in /docs/reference/pytxo-toml exist in
  `crates/pytxo-core/src/config.rs` and related config; the 14 MCP tools in
  /docs/reference/mcp-tools match `crates/pytxo-mcp/src`; every Desktop label
  the setup, first-mission and review guides tell users to click exists in
  `apps/desktop/src`.
- **Site health:** 63 sitemap URLs return 200; every page has a unique title
  (after #13), description, canonical and OG image; no `noindex` on public
  pages; robots and sitemap valid; docs search API returns results.
- **Mobile:** no horizontal scroll at 375 px on any of the 65 pages; tap
  targets pass (small text links have 24 px spacing or sit inline).
- **Accessibility:** no images without alt, no unnamed controls, visible focus
  on the first ten tab stops of home, docs, download and plans, `lang="en"`,
  reduced motion honoured in the hero, apply-gate and ASCII art.
- **External links:** all 23 return 200 except npm and BridgeMind, which
  return 403 only to non-browser clients (200 with a browser user agent).
- **Legal:** Terms, Privacy and Refunds agree with Plans (Dodo Payments as
  merchant of record, 14-day first-payment refund, checkout not yet open).

## Not tested

- Desktop MSI first run (native GUI; the guides were checked against source
  instead).
- Sign-in, account, receipt sharing and Pro checkout: production accounts are
  out of bounds for this review, and checkout is closed.

## Checks run on the fixes

All in `apps/web`, on the working tree with every fix above:

- `pnpm lint`: clean.
- `pnpm check:links`: 221 internal links across 147 files, none broken.
- `pnpm build`: passes (78 static pages, 56 docs pages).
- `npx playwright test` against `next start` on the build: 32/32 passed.
- A one-off Playwright check of each fix against the same build, 25/25:
  mobile menu closes after a link tap; no horizontal scroll at 375 px on
  /, /plans, /download, /bench, the 404 and /docs/troubleshooting; unknown
  URLs and unknown docs pages return 404 with title "Page not found · Pytxo",
  one `noindex` robots tag and links out; "Skip to content" is the first tab
  stop, visible, and lands on `#main`; logo link reads "Pytxo" once; plans
  headings go h1 → h2; inline links are underlined; footer legal row is
  `rgb(125,125,135)`; /bench title is unique; the macOS/Linux copy shows;
  evidence pages log no console 404s and the JSON links still open the file;
  troubleshooting no longer names v1.2.2; the Warp link and `project remove`
  row are in place.
- Desktop (1440) and mobile (375) screenshots of the 404, Download, Plans,
  Pybench and the focused skip link were reviewed; nothing else on those pages
  moved.
