<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/SCREEN_SPECIFICATIONS_MERCHANT.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Merchant Application Screen Specifications

> **Document role:** Normative UX/visual specification.

## 1. Merchant home

Show:
- receive payment action
- connectivity state
- pending synchronization
- latest local transactions

The screen should optimize for speed and verification.

## 2. Request generation

Support:
- amount
- merchant identity
- QR representation
- NFC representation where supported

## 3. Payment received

Show:
- amount
- payer reference
- verification state
- local storage state
- reconciliation status

Do not show a generic "settled" label unless backend reconciliation actually confirms it.

## 4. Pending reconciliation

Show:
PAYMENT RECEIVED LOCALLY

and:
RECONCILIATION PENDING

Explain that local acceptance and final reconciliation are different states.

## 5. Transaction history

Prioritize:
- timestamp
- amount
- counter/reference
- state
- synchronization state

Use compact rows with strong separators.

## 6. Conflict

A reconciliation conflict should be visually distinct from a routine network delay.

Show:
- transaction reference
- conflict state
- recommended action
- whether retry is allowed

Do not ask the merchant to manually alter transaction data.
