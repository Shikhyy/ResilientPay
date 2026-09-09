<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/LANDING_PAGE.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Landing Page Specification

> **Document role:** Normative UX/visual specification.

## Objective

The landing page is a product and research narrative. It should help a technically aware visitor understand what ResilientPay is, why it exists, how it works, what was actually built, and what evidence supports the claims.

This is not a generic SaaS conversion page.

## Page architecture

1. Header
2. Hero
3. Connectivity problem
4. What ResilientPay is
5. Live product demonstration
6. System architecture
7. Multi-modal transport explanation
8. Transaction state journey
9. Security model
10. Reconciliation
11. Research evidence and experiments
12. Deployment boundary
13. Documentation
14. Legal and trust
15. Footer

## 1. Header

Left:
RESPAY / RESEARCH PROTOTYPE

Primary navigation:
System
Demo
Protocol
Research
Documentation

Secondary action:
Contact

The header uses a strong bottom rule. No floating rounded navigation bar.

## 2. Hero

Layout: asymmetric two-column composition.

Left side:
Eyebrow:
CONNECTIVITY-RESILIENT PAYMENT RESEARCH

Headline:
Payments designed for unreliable connectivity.

Body:
ResilientPay is a research prototype exploring bounded offline authorization, multi-modal transaction transfer, and eventual reconciliation across degraded network conditions.

Actions:
EXPLORE THE SYSTEM
SEE THE LIVE DEMO

Right side:
A live or animated Signal Line diagram showing:
Internet
Proximity
Local storage
Synchronization

Below the diagram, a compact system metadata panel:
TRANSPORT / NFC · BLE · QR · SMS · INTERNET
SECURITY / SIGNED TRANSACTION ENVELOPES
SETTLEMENT / EVENTUAL RECONCILIATION

The hero must show the actual system concept immediately.

## 3. Connectivity problem

Headline:
When the network disappears, the transaction still has to make sense.

Show a large horizontal degradation diagram:

CONNECTED
    ↓
LOW CONNECTIVITY
    ↓
PROXIMITY AVAILABLE
    ↓
LOCAL STORAGE
    ↓
RECONCILIATION

Explain that the project studies how transaction integrity can be preserved as communication options degrade.

## 4. What ResilientPay is

Use a large editorial statement followed by a technical diagram.

Statement:
A transport-independent payment architecture for bounded offline operation and store-and-forward recovery.

Then describe:
- payer device
- merchant device
- transaction protocol
- local ledger
- reconciliation service
- risk engine

Do not use three feature cards.

Use a numbered vertical system index.

## 5. Product demonstration

This section is mandatory.

Header:
SEE THE SYSTEM UNDER FAILURE

The demonstration should let the visitor:
- start with internet available
- disable internet
- select NFC, BLE, or QR
- create a bounded offline payment
- inspect the resulting transaction state
- restore connectivity
- trigger synchronization
- observe reconciliation

Provide an optional "attack mode" in the research environment:
- replay
- duplicate submission
- altered amount
- invalid signature

The landing page should link to the full demo if the full simulation cannot run inline.

## 6. System architecture

Show the actual architecture, not a decorative dashboard.

Payer App
→ Payment Engine
→ Transport Adapter
→ Merchant App
→ Local Ledger
→ Reconciliation API
→ Authoritative Backend

Add side rails for:
Crypto
Risk
Audit
Observability

## 7. Multi-modal transport

Use a transport matrix:

| Transport | Connectivity assumption | Strength | Constraint |
|---|---|---|---|
| Internet | full network | normal connected operation | unavailable during outage |
| NFC | physical proximity | short local exchange | requires supported hardware |
| BLE | local radio | flexible proximity transport | pairing/discovery complexity |
| QR | optical proximity | broad compatibility | human/device interaction constraints |
| SMS | telecom path | store-and-forward channel | delayed, duplicated, reordered |

Do not call SMS "offline payment". Explain it as a telecom-assisted recovery transport.

## 8. Transaction state journey

Use a large interactive timeline:

CREATED
→ AUTHORIZED
→ SIGNED
→ TRANSFERRED
→ RECEIVED
→ STORED
→ SYNC_PENDING
→ RECONCILED

Each state expands into a short definition.

The final state must not appear until reconciliation is actually confirmed.

## 9. Security

Present five layers:
Credential
Signature
Counter
Ledger
Reconciliation

Each layer has one concise explanation.

Example:
COUNTER
Protects against replay of an already-used authorization sequence.

Do not show secrets, keys, or internal security material.

## 10. Reconciliation

Show a real synchronization sequence:

LOCAL EVENT
→ BATCH
→ VERIFY
→ IDEMPOTENCY
→ CONFLICT CHECK
→ ACCEPT / REJECT / PENDING
→ AUDIT

Add a small example of duplicate submission handling.

## 11. Research evidence

Use measured experiments:
- transport reliability
- reconciliation latency
- duplicate detection
- replay resistance
- risk-model performance
- storage overhead
- serialization size

Each metric should show:
Question
Method
Result
Limitations

No invented numbers.

## 12. Deployment boundary

Clearly label:
RESEARCH PROTOTYPE

Explain that the prototype does not itself claim authorization to operate a regulated payment system or move real customer funds.

## 13. Documentation

Expose:
Architecture
Protocol
Security
Experiments
API
Developer guide

Use a documentation index, not a generic resources grid.

## 14. Legal and trust

Include:
Terms of Service
Privacy Policy
Prototype Disclaimer
Security Disclosure
Contact

These links must be visible in the footer.

## 15. Footer

A dense editorial footer with a top border. No oversized decorative social icon area.
