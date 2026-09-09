<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/DESIGN_CHANGE_CONTROL.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Design Change Control

> **Document role:** Normative UX/visual specification.

## Why

Visual consistency is part of product correctness. Agents should not gradually change the design language through individually reasonable but collectively inconsistent additions.

## Change classes

### Class A: token change
Examples:
- color value
- typography scale
- spacing token

Requires:
design review and screenshot comparison.

### Class B: component change
Examples:
- new status component
- new payment control

Requires:
component specification and accessibility review.

### Class C: page composition
Examples:
- landing page section order
- new interaction model

Requires:
UX review and content review.

### Class D: product-state representation
Examples:
- changing "reconciliation pending" wording
- changing status semantics

Requires:
protocol/product review before design approval.

## Prohibited shortcut

No agent may change a domain-state label for visual convenience.

## Traceability

Every significant design change should reference:
- issue/task
- relevant screen specification
- component
- protocol state if applicable
- screenshots
- reviewer
