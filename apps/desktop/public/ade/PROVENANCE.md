# Harness identity assets

Checked 2026-09-14. These marks identify the named local harness beside its
full text label. They do not indicate installation, authentication,
verification, partnership, or endorsement. Vendor names and marks remain the
property of their respective owners.

All 14 rendered assets have a transparent canvas. Pytxo uses each mark only
beside its full product name; identity does not imply installation, readiness,
partnership, or endorsement.

| Pytxo id | Local asset | Source and adaptation |
| --- | --- | --- |
| `claude` | `anthropic.svg` | Simple Icons `anthropic.svg`, commit `4ba19240849175ab4b855a732ab98c0f87cfb714`, sourced there from Anthropic's official site. Rendered as a current-color mask. |
| `agy` | `antigravity.png` | Exact `Icon — Full Color` PNG from Google Antigravity's official press page, retrieved 2026-09-15; SHA-256 `E0CD08CCD10CD8D08CCF0BA449823EE88495825C0841619618100D3AB089F51E`. The complete 540×540 transparent file and its clear space are preserved. This local review asset must be removed from a public build unless Google's required compatibility-use approval is recorded. |
| `codex` | `openai-on-dark.svg`, `openai-on-light.svg` | Exact mono Blossom files from OpenAI's official `OpenAI-Logos-2025.zip`; archive SHA-256 `B2C4CD1E86BBE76BDC4946A72D014EFA455240C177F4878FD68CC9B88C71D2EC`. Theme variants are switched without recoloring or geometry changes. |
| `cursor` | `cursor-on-dark.svg`, `cursor-on-light.svg` | Exact `CUBE_2D_DARK.svg` and `CUBE_2D_LIGHT.svg` from Cursor's official brand kit; archive SHA-256 `97488A7751914E60F9FF532BC33810CDEAEBDDDC017548ABE6CA2BC29BBC3928`. |
| `opencode` | `opencode.svg` | `anomalyco/opencode`, commit `df23b7f9488a38e6f8064a0739d4f8cde86d7cfb`, `packages/console/app/src/asset/brand/opencode-logo-dark-square.svg`; MIT. |
| `gemini` | `gemini.svg` | Simple Icons `googlegemini.svg`, commit `4ba19240849175ab4b855a732ab98c0f87cfb714`, sourced there from Gemini's official site. Rendered as a mask using the catalogued `#8E75B2` brand color. |
| `copilot` | `copilot.svg` | GitHub Primer Octicons `copilot-24.svg`, commit `0b52df259a3e4df4396f7741e53438e9a022d46f`; MIT. Rendered as a current-color mask. |
| `aider` | `aider.png` | Exact transparent `favicon-32x32.png` from `Aider-AI/aider`, commit `5dc9490bb35f9729ef2c95d00a19ccd30c26339c`; SHA-256 `76DF46F16497ADAAF724C98DB79A6E32058F84DC8DE384A35D2A7D38F7437BBB`; Apache-2.0. Rendered unchanged as an image. |
| `grok` | `grok-on-dark.svg`, `grok-on-light.svg` | Exact transparent `Grok_Logomark_Light.svg` and `Grok_Logomark_Dark.svg` from SpaceXAI's official brand archive; archive SHA-256 `DB9129ACD4EFC4C2202D25AFE31B70281A79F8507F75520AB5E6B3356895A7E9`. File hashes are `B20648E2F111D7FBC91F58B22D1E76E9885B68A163CB5A1010F7F11BF5840491` and `A127A7CD42B0450F7D3827A331B0730AAB49FD99C3FE920D172475B9FFC83992`. Theme variants are switched without altering their bytes. |
| `droid` | `factory-droid.svg` | Foreground geometry from Factory's official `https://factory.ai/favicon.svg`, retrieved 2026-09-14. The opaque black canvas was removed and the foreground is rendered as a current-color mask. |
| `cline` | `cline.svg` | Transparent robot mark from `cline/cline`, commit `94980446c99f24040e9ed7a03e7726be4aea9198`, `apps/cline-hub/src/webview/public/cline-logo-filled.svg`; upstream SHA-256 `C38496CF76F8106D626DC02E62FDEE45B4341F1C1EE51C53BC13F5878605F598`, local SHA-256 `E1D26E2744DACF0211F459A2B542C7266B648F866FB7BB8EE85C6A579BB59E81` after adding the repository-standard trailing newline; Apache-2.0. The path geometry is unchanged and renders as a current-color mask so the mark remains legible in both Pytxo themes. |
| `goose` | `goose.svg` | `block/goose`, commit `50666ae0b9a51e260b52b7efbab2e4e020346e94`, `documentation/static/img/logo.svg`; Apache-2.0. |
| `qwen` | `qwen.svg` | `QwenLM/qwen-code`, commit `87437db784f6ffbdb725b46b4a5fdd5fbac9cd74`, `packages/desktop-shell/bootstrap/qwen-code-logo.svg`; Apache-2.0. |
| `kimi` | `kimi.svg` | `MoonshotAI/kimi-code`, commit `f37cb3d18c98dbd3d4c38838c4b320eedf28a5ec`, `apps/vscode/resources/kimi-icon.svg`; MIT. Rendered as a current-color mask. |

`cursor.ico`, `gemini.ico`, and `opencode.png` are legacy inputs retained for
handoff safety but are no longer referenced by the component because their
visible tiles conflict with the transparent compatibility-mark treatment.

## Publication boundary

The Aider and Grok files are narrow identifying uses with the full adjacent
product name. Grok's files remain byte-exact as required by SpaceXAI's published
brand terms. Google's current product-icon guidance requires a Partner Marketing
Hub approval request for a compatibility use. `antigravity.png` is therefore a
local review asset, not approved public-release artwork; remove it or record the
approval before distributing a build that contains it. See
`docs/01-projects/pytxo-final-three-harness-logo-research-2026-09-15.md`.

Simple Icons publishes its collection under CC0-1.0 while explicitly warning
that individual brand rights may differ. See the pinned upstream
`LICENSE.md` and `DISCLAIMER.md`. This use is limited to identifying compatible
services beside unambiguous text labels.
