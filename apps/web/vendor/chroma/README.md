# @pytxo/chroma

Shared Pytxo Chroma ribbon tokens and utility classes.

Palette: Void / Panel / Ink, plus a logo-spectrum ribbon. Live (running) and Cue (needs-you / Apply) are status only.

## Files

- `tokens.css` — CSS custom properties (void night + light)
- `utilities.css` — ribbon, bezel, status lamp, Apply receipt
- `tokens.json` — machine-readable palette for Rust TUI sync

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
