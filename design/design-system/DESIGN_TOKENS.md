<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/DESIGN_TOKENS.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Design Tokens

> **Document role:** Normative UX/visual specification.

## Color tokens

Use a warm document-like base rather than pure white.

| Token | Value | Use |
|---|---|---|
| paper | #F1EEE7 | primary page background |
| surface | #E4E0D7 | panels, secondary surfaces |
| ink | #202522 | primary text |
| slate | #5E6763 | secondary text |
| rule | #A8AAA3 | borders and separators |
| teal | #146B68 | active system state, links, verified state |
| orange | #C75A24 | actions, warning, degraded connectivity |
| red | #A53B32 | security rejection and critical failure |
| blue | #385C71 | informational technical data |

Do not use gradients. Do not introduce arbitrary brand colors without documenting their semantic purpose.

## Typography tokens

Primary:
- IBM Plex Sans

Technical:
- IBM Plex Mono

Editorial:
- Source Serif 4

Do not use Inter, Geist, or Space Grotesk.

Suggested scale:

| Token | Desktop | Mobile |
|---|---:|---:|
| display | 64px | 42px |
| h1 | 48px | 36px |
| h2 | 36px | 30px |
| h3 | 25px | 22px |
| body | 17px | 16px |
| small | 14px | 13px |
| mono | 13px | 12px |

Adjust by content, not by trend.

## Geometry

Default radius: 0px.
Maximum normal radius: 2px.

Use 1px rules for structural boundaries and 2px rules for major emphasis.

## Spacing

Use a consistent 4px base with larger editorial increments.

Core values:
4, 8, 12, 16, 24, 32, 48, 64, 96, 128.

## Motion

Motion should be short and purposeful:
- 120 to 180ms for interaction feedback
- 180 to 280ms for panel transitions
- longer only for meaningful protocol simulations

Respect prefers-reduced-motion.

## Iconography

Use a custom geometric icon set. Icons should be visually related to:
- devices
- signal
- transfer
- storage
- synchronization
- security
- transport

Avoid generic consumer-finance icons.
