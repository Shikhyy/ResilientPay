<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/DESIGN_TO_CODE.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Design to Code Rules

> **Document role:** Normative UX/visual specification.

## Source hierarchy

Approved design tokens
→ approved components
→ screen specifications
→ implementation

## Agent behavior

Before creating UI:
1. read root AGENTS.md
2. read this design directory's README
3. locate the relevant screen specification
4. locate an existing component
5. check whether a design token already exists
6. implement the smallest compliant change
7. run visual and functional tests
8. update documentation if the component contract changes

## Prohibited implementation shortcuts

- arbitrary colors
- arbitrary border radii
- generic icon packages
- fake loading states
- missing error states
- invented product claims
- placeholder screenshots presented as product evidence
- hard-coded protocol states not defined by the domain model

## State binding

UI status must map to domain state.

For example:
SYNC_PENDING must be sourced from domain state, not inferred from whether a network request happens to be running.

## New component protocol

A new component requires:
- rationale
- anatomy
- states
- accessibility
- responsive behavior
- screenshot coverage

## Review

A UI change is incomplete until:
- visual regression passes
- accessibility passes
- domain-state mapping is reviewed
- forbidden design-pattern scan is clean
