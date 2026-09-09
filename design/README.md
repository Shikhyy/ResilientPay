<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/README.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# ResilientPay Design & Visual System

> **Document role:** Normative UX/visual specification.

This directory is the visual and product-design source of truth for the ResilientPay project.

ResilientPay is a research-grade connectivity-resilient payment platform. The interface must communicate technical credibility, physical-world constraints, payment state, security, and research evidence without falling into generic fintech or AI-generated SaaS aesthetics.

## Design objective

The product should feel like a serious engineering system that happens to have a polished user interface.

The visual language is based on:
- engineering documentation
- telecommunications infrastructure
- physical payment devices
- transport diagrams
- editorial research publications
- institutional software
- precise status communication

The design is intentionally restrained. Decoration must earn its place by improving comprehension.

## Hard constraints

The following are prohibited across the website and applications:

- harsh gradients
- Lucide or equivalent generic icon-library usage as the visual identity
- pure white backgrounds as a primary surface
- rainbow palettes
- drop shadows
- exactly three feature cards in a row
- emojis in UI or marketing copy
- liquid glass or glassmorphism
- em dashes in copy
- Inter, Geist, or Space Grotesk
- colored left stripes on cards or alerts
- fake testimonials or fabricated customer evidence
- bento-grid layouts
- fake terminal windows
- the copy formula "it's not X, it's Y"
- checkmark bullets as a visual motif
- standard three-tier pricing layouts
- skipping the real product demonstration
- soft generic corner radiuses
- purple-and-black visual themes
- missing skeleton loading states
- radial background orbs
- dot-grid backgrounds
- sparkle icons
- animated arrows
- omitted Terms of Service
- omitted Privacy Policy
- generic hover-animation patterns
- neon colors
- basic pastel palettes

The project may use small radii of 0 to 2px when needed for platform behavior, but the default geometry is sharp.

## Source-of-truth hierarchy

1. Product and protocol specifications define what the system actually does.
2. This design directory defines how those capabilities are communicated and interacted with.
3. Figma or equivalent visual files represent composition and final visual calibration.
4. Application code implements approved design decisions.
5. Screenshot tests, accessibility tests, and review validate the implementation.

A visual treatment must never invent a capability or contradict a protocol state.


## Repository relationship

Design specifications are normative for visual and interaction behavior. Implementation must not invent visual states that contradict domain or protocol state.

## Design review loop

```text
UX requirement
→ flow
→ wireframe
→ visual specification
→ component
→ implementation
→ visual regression
→ accessibility verification
→ protocol-state verification
→ human review
→ merge
```
