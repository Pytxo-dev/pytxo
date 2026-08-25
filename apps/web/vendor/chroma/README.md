# @pytxo/chroma

Shared Pytxo Chassis design tokens and utility classes.

Palette: Bezel / Plate / Aluminum metal, plus Live (running) and Cue (needs-you / Apply).

## Files

- `tokens.css` — CSS custom properties (Chassis + semantic dark/aluminum)
- `utilities.css` — bezel, status lamp, Apply receipt
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
