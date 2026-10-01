---
title: Pytxo final three harness logo research, September 2026
slug: pytxo-final-three-harness-logo-research-2026-09-15
status: active
tags: [project, desktop, harnesses, research, brand]
audience: [human, agent]
layer: product
created: 2026-09-15
updated: 2026-09-15
related: [[pytxo-harness-catalog-research-2026-09-14]]
---

# Pytxo final three harness logo research, September 2026

## Decision summary

| Harness | Transparent first-party mark | Safe Pytxo treatment | Decision |
| --- | --- | --- | --- |
| Google Antigravity | Yes: three 540×540 transparent PNG icons on the official press page | Preserve the complete supplied PNG, including its transparent clear space, at 24px. Do not recolor, crop, trace, mask, or combine it with Pytxo branding. | **Technically ready, public-use approval unresolved.** Google's current rules require an approval request for a product icon used to show product compatibility. |
| Aider | Yes: a 32×32 transparent favicon in the official repository and live site | Use the exact PNG at 20px with normal image sampling; do not extract or redraw its pixel-art glyph. Keep “Aider” as the adjacent identifying text. | **Ready with notice and narrow identifying use.** Apache-2.0 covers the repository asset, but does not grant broader trademark rights. |
| Grok Build | Yes: the official SpaceXAI archive includes transparent black and white Grok logomark SVGs | Render the exact white SVG on dark surfaces and exact black SVG on light surfaces as external images at 20–24px. Do not edit paths, recolor, mask, crop, or add effects. | **Ready under the published brand terms.** Use only to accurately identify Grok Build and never imply endorsement. |

## Google Antigravity

Google's [Antigravity press-assets page](https://antigravity.google/press) now provides three icon-only downloads:

- [Icon — Full Color](https://antigravity.google/assets/image/brand/antigravity-icon__full-color.png): 540×540 ARGB PNG, transparent corners, 71,914 bytes, SHA-256 `E0CD08CCD10CD8D08CCF0BA449823EE88495825C0841619618100D3AB089F51E`.
- [Icon — One Color](https://antigravity.google/assets/image/brand/antigravity-icon__one-color.png): 540×540 ARGB PNG, transparent corners, 8,662 bytes, SHA-256 `00B50E4F0243CD07DCF536CF36BB2CF071C3E0445A99CA9AFCB90142F870CC01`.
- [Icon — White](https://antigravity.google/assets/image/brand/antigravity-icon__white.png): 540×540 ARGB PNG, transparent corners, 7,437 bytes, SHA-256 `B65FAC00147E2402DCEF1D7730AD0FEEC6611C6ED5C5CB2EAD64AD7609190405`.

The full-color icon is the strongest small-size identifier and already works on both Pytxo themes. Render the entire file at 24px so its built-in clear space remains intact. The monochrome files are exact fallbacks, not raw shapes to recolor.

Asset availability is not sufficient permission for a public compatibility UI. Google's [brand-element guidance](https://about.google/brand-resource-center/brand-elements/#product-icons) says product-icon use requires a Partner Marketing Hub account and approval request. Its [product co-branding guidance](https://partnermarketinghub.withgoogle.com/brands/google/use-cases/product-co-branding/#showing-product-compatibility) describes a compatibility grid as a legitimate context, but still requires Google to remain secondary, the icon to be necessary and recognizable in context, and legal attribution. Until approval is recorded, keep the public/release treatment text-only or explicitly gate the asset from distribution.

Separate from artwork permission, the current [Antigravity Additional Terms](https://antigravity.google/terms) prohibit third-party tools from accessing the service in connection with products not provided by Google. Pytxo should confirm that its user-installed CLI launch model is permitted before publicly claiming runnable integration; the presence of a logo must not imply that review has passed.

## Aider

The official `Aider-AI/aider` repository currently points to commit [`5dc9490bb35f9729ef2c95d00a19ccd30c26339c`](https://github.com/Aider-AI/aider/tree/5dc9490bb35f9729ef2c95d00a19ccd30c26339c). Its first-party website assets contain:

- [Transparent 32×32 favicon PNG](https://raw.githubusercontent.com/Aider-AI/aider/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/website/assets/icons/favicon-32x32.png): 1,891 bytes, SHA-256 `76DF46F16497ADAAF724C98DB79A6E32058F84DC8DE384A35D2A7D38F7437BBB`. The [live copy](https://aider.chat/assets/icons/favicon-32x32.png) matched byte-for-byte on 2026-09-15.
- [Official wordmark SVG](https://raw.githubusercontent.com/Aider-AI/aider/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/aider/website/assets/logo.svg): 200×60, 5,742 bytes, SHA-256 `C2012BA0A3141EF162F6BC6F4CC8CB4AB19444DBA0408C975F58CEEDA4ED79B1`. It embeds a base64 font and glow filter, so it is unsuitable for a 20–24px catalog slot.

Use the favicon PNG unchanged. It has a transparent canvas (`Format32bppArgb`; corner alpha 0) and a high-contrast green pixel mark. Do not convert the wordmark into a guessed icon.

The repository's [Apache-2.0 license](https://github.com/Aider-AI/aider/blob/5dc9490bb35f9729ef2c95d00a19ccd30c26339c/LICENSE.txt) permits redistribution of the asset subject to the license and retained notices, but section 6 does not grant trademark permission beyond reasonable, customary source identification. Pytxo's narrow catalog use should therefore remain plainly descriptive, with Aider adjacent in text and no suggestion that the mark is Pytxo-owned or that Aider endorses Pytxo. Preserve the Apache notice; request maintainer clarification before broader promotional use.

## Grok Build

SpaceXAI's [brand guidelines](https://x.ai/legal/brand-guidelines) link the official [SpaceXAI/Grok asset archive](https://data.x.ai/logos/SpaceXAI_Grok_Assets.zip). The archive downloaded successfully with a browser user agent and `https://x.ai/legal/brand-guidelines` referrer on 2026-09-15:

- ZIP: 351,396 bytes, SHA-256 `DB9129ACD4EFC4C2202D25AFE31B70281A79F8507F75520AB5E6B3356895A7E9`.
- `Grok_Logomark_Dark.svg`: transparent 1024×1024 vector with black `#0A0A0A` paths, 965 bytes, SHA-256 `A127A7CD42B0450F7D3827A331B0730AAB49FD99C3FE920D172475B9FFC83992`.
- `Grok_Logomark_Light.svg`: transparent 1024×1024 vector with white paths, 961 bytes, SHA-256 `B20648E2F111D7FBC91F58B22D1E76E9885B68A163CB5A1010F7F11BF5840491`.

The SVGs contain only the supplied paths on a transparent canvas; there is no background rectangle. For Pytxo's dark theme use `Grok_Logomark_Light.svg`; for its light theme use `Grok_Logomark_Dark.svg`. Keep both files byte-exact and switch the external image source by theme. CSS should only size/position the whole file (`object-fit: contain`); filters, masks, path extraction, recoloring, animation, or a custom surrounding badge would violate the published instruction to use logos exactly as provided without alteration or adjustment.

The same guidelines permit accurate reference to SpaceXAI/Grok, prohibit misleading endorsement or sponsorship, prohibit incorporating the mark into Pytxo's own identity, and reserve SpaceXAI's right to require changes or terminate permission. A small compatibility mark beside the truthful “Grok Build” label fits the stated reference purpose; retain the source URL and brand terms in Pytxo's provenance/third-party notices.

## Implementation result

The bounded local-review implementation now uses the exact Aider PNG, the two
byte-exact Grok SVG variants, and the exact full-color Antigravity PNG. The
Desktop identity map and regression coverage now include all 14 recognized
harnesses, with Grok switching between the supplied light and dark marks.

This does **not** clear the Antigravity asset for publication. Any publicly
distributed candidate must either record the required Google compatibility-use
approval and attribution or omit that mark. Native inspection at the final
20–24px size remains part of the candidate acceptance record.
