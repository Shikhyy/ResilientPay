<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/ACCESSIBILITY.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Accessibility

> **Document role:** Normative UX/visual specification.

Accessibility is part of design correctness.

## Requirements

- semantic HTML where applicable
- keyboard navigation
- visible focus state
- sufficient text contrast
- no color-only status communication
- descriptive labels for system states
- screen-reader accessible loading and error messages
- motion reduction support
- adequate touch targets
- correct heading hierarchy
- meaningful table headers

## Connectivity state

Do not communicate offline status using a color dot alone.

Use:
OFFLINE
or
SYNC PENDING

with semantic markup.

## Transactions

A screen reader should be able to announce:
"Payment of 240 rupees. Merchant Demo Store. Authorized locally. Reconciliation pending."

Do not expose sensitive keys or secrets.

## Testing

Run automated accessibility checks plus manual keyboard and screen-reader review for critical flows.
