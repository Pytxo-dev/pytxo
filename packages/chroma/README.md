# @pytxo/chroma

Shared Pytxo Chroma design tokens and utility classes.

## Files

- `tokens.css` — CSS custom properties (brand + semantic dark/light)
- `utilities.css` — Chroma utility classes (nebula, glass-panel, chroma-border, etc.)
- `tokens.json` — Machine-readable palette for Rust TUI sync

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
