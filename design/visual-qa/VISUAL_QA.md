<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/VISUAL_QA.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Visual QA and Review

> **Document role:** Normative UX/visual specification.

## Review levels

### Level 1: automated

Check:
- forbidden CSS patterns
- design token usage
- accessibility
- responsive overflow
- missing loading states
- missing alt text where applicable

### Level 2: screenshot comparison

Capture critical screens at:
- mobile
- tablet
- desktop

Compare:
- typography
- spacing
- alignment
- borders
- state representation
- loading
- error
- empty states

### Level 3: human review

Review:
- product truthfulness
- hierarchy
- information density
- technical clarity
- visual consistency
- accessibility
- protocol-state accuracy

## Design drift checklist

Reject if the implementation introduces:
- gradients
- pure white primary background
- generic rounded cards
- shadows
- generic icon packs
- purple-heavy palette
- neon color
- decorative blobs
- dot grids
- fake testimonials
- fake terminal UI
- exactly three feature cards as a default marketing pattern
- standard pricing tiers
- missing TOS or Privacy Policy
- missing loading states

## Release gate

No production-facing visual release should occur without a screenshot set and a completed review note.
