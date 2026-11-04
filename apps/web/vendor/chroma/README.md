# @pytxo/chroma

Shared Pytxo Chroma ribbon tokens and utility classes.

Palette: Void / Panel / Ink, plus a spectrum ribbon. Live (running) and Cue (needs-you / Apply) are status only.

## Files

- `tokens.css` — CSS custom properties (void night + light)
- `utilities.css` — ribbon, bezel, status lamp, Apply receipt
- `tokens.json` — machine-readable palette for Rust TUI sync
- `logo/pytxo-mark.svg` — master logo mark (merge λ): a λ whose strokes meet at a magenta node, the merge where Pytxo runs its check. The λ is Ink, for dark backgrounds; `logo/pytxo-mark-light.svg` is the same mark in Void for light ones. `apps/web/public/logo.png` is rendered from the dark mark at 1024 px, and `tooling/scripts/generate-deck-icons.mjs` derives `logo-mark.png` (site header and footer) from that PNG.
- `logo/pytxo-icon.svg` — the mark on a Void tile, for app icons and favicons, which have to read on light and dark taskbars and tabs. `logo/pytxo-icon-16.svg` is the tile drawn on the 16 px grid. `tooling/scripts/generate-deck-icons.mjs` renders the Desktop bundle icons from the two tiles, and `apps/web/scripts/generate-favicon.mjs` the favicons.

## Usage

### Web (Next.js)

```css
@import "@pytxo/chroma/tokens.css";
@import "@pytxo/chroma/utilities.css";
```

### Desktop (Svelte + Vite)

```css
@import "../../../packages/chroma/tokens.css";
@import "../../../packages/chroma/utilities.css";
```

### TUI (Rust)

Constants in `crates/pytxo-tui/src/theme.rs` must match `tokens.json`. Run `cargo test -p pytxo-tui theme::` to verify alignment.
