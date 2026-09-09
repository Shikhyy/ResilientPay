<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/SCREEN_SPECIFICATIONS_PAYER.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Payer Application Screen Specifications

> **Document role:** Normative UX/visual specification.

## 1. Home

Primary purpose: show current payment capability and recent activity.

Must show:
- connectivity state
- available payment action
- pending reconciliation count
- recent transactions

Loading:
Skeleton for balance/status and recent transactions.

Empty:
Explain that no transactions are stored locally.

## 2. Amount entry

Must show:
- amount
- currency
- merchant/payee identity
- available payment method

Never expose:
- risk score
- cryptographic identifiers not meaningful to the user

Validation:
- reject zero and invalid amounts
- use integer minor units internally
- show precise validation messages

## 3. Payment method

Methods may include:
- Online
- NFC
- BLE
- QR

SMS is not a proximity payment control. It is a synchronization/recovery mechanism.

The UI should show why a method is unavailable.

## 4. Authorization

Show:
- recipient
- amount
- authorization request
- appropriate device authentication

Do not imply settlement.

## 5. Local authorization result

If the transaction is stored locally:

PRIMARY STATUS:
AUTHORIZED LOCALLY

SECONDARY:
Stored securely. Reconciliation is pending.

## 6. Transaction detail

Show:
- amount
- recipient
- transaction reference
- local state
- creation time
- transport
- synchronization state
- reconciliation state

## 7. Sync pending

Use a clear timeline:
LOCAL
STORED
WAITING FOR CONNECTIVITY
SYNCING
RECONCILED

Provide retry only where technically meaningful.

## 8. Security rejection

Title:
Payment could not be verified.

Explain the next safe action.

Do not disguise a security rejection as a generic network failure.
