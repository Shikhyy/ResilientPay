<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/LANDING_PAGE_WIREFRAME.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Landing Page Wireframe

> **Document role:** Normative UX/visual specification.

## Desktop wireframe

```text
┌─────────────────────────────────────────────────────────────────────┐
│ RESPAY / RESEARCH          SYSTEM  DEMO  PROTOCOL  RESEARCH  DOCS │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ CONNECTIVITY-RESILIENT PAYMENT RESEARCH       ┌──────────────────┐ │
│                                                │ SIGNAL LINE      │ │
│ Payments designed for                         │                  │ │
│ unreliable connectivity.                      │ CONNECTED        │ │
│                                                │ LOCAL            │ │
│ ResilientPay is a research                    │ STORE            │ │
│ prototype exploring...                         │ SYNC             │ │
│                                                └──────────────────┘ │
│ [EXPLORE SYSTEM] [SEE LIVE DEMO]                                   │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ WHEN THE NETWORK DISAPPEARS,                                       │
│ THE TRANSACTION STILL NEEDS A RELIABLE ANSWER.                     │
│                                                                     │
│      CONNECTED → LOCAL → STORE → SYNC                              │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ SEE THE SYSTEM UNDER FAILURE                                       │
│                                                                     │
│ ┌─────────────── DEMO ──────────────────────────────────────────┐  │
│ │ Connectivity: OFFLINE     Transport: NFC                     │  │
│ │ Amount: ₹240              State: AUTHORIZED LOCALLY            │  │
│ │                                                              │  │
│ │ PAYER ─── NFC ─── MERCHANT ─── LOCAL LEDGER                 │  │
│ │                                                              │  │
│ │ [RESTORE CONNECTIVITY]                                       │  │
│ └───────────────────────────────────────────────────────────────┘  │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ ONE TRANSACTION MODEL. MULTIPLE WAYS TO MOVE IT.                   │
│                                                                     │
│ Internet / NFC / BLE / QR / SMS                                    │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ TRANSACTION STATE JOURNEY                                          │
│ CREATED → AUTHORIZED → SIGNED → TRANSFERRED → RECEIVED →          │
│ STORED → SYNC_PENDING → RECONCILED                                  │
├─────────────────────────────────────────────────────────────────────┤
│ SECURITY / RECONCILIATION / RESEARCH EVIDENCE                     │
├─────────────────────────────────────────────────────────────────────┤
│ DOCUMENTATION / TERMS / PRIVACY / SECURITY                         │
└─────────────────────────────────────────────────────────────────────┘
```

## Mobile wireframe

```text
RESPAY
RESEARCH PROTOTYPE
───────────────

CONNECTIVITY
OFFLINE

Payments designed for
unreliable connectivity.

[ SEE LIVE DEMO ]

────────────────

CURRENT CAPABILITY
NFC
BLE
QR

────────────────

LIVE DEMONSTRATION

STATE
AUTHORIZED LOCALLY

RECONCILIATION
PENDING

────────────────

HOW IT WORKS

1  Authorize
2  Transfer
3  Store
4  Sync
5  Reconcile

────────────────

PROTOCOL
SECURITY
RESEARCH
DOCUMENTATION
```
