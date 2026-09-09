<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/LANDING_PAGE_COPY.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Landing Page Copy

> **Document role:** Normative UX/visual specification.

## Hero

### Eyebrow
CONNECTIVITY-RESILIENT PAYMENT RESEARCH

### Headline
Payments designed for unreliable connectivity.

### Body
ResilientPay is a research prototype exploring how bounded offline authorization, local transaction exchange, and eventual reconciliation can preserve payment integrity when conventional connectivity is unavailable.

### Primary action
EXPLORE THE SYSTEM

### Secondary action
SEE THE LIVE DEMO

## Problem section

### Headline
When communication fails, payment state still needs a reliable answer.

### Body
A payment can move through more than one communication path. ResilientPay studies how a transaction can be authenticated locally, transferred through available proximity channels, retained safely, and reconciled when connectivity returns.

## System section

### Headline
One transaction model. Multiple ways to move it.

### Body
The same logical payment object can move through internet, NFC, Bluetooth, QR, or store-and-forward SMS transport. Transport determines how information travels. It does not determine whether the payment is valid.

## Demo section

### Headline
See what happens when the network disappears.

### Body
Start connected. Disable the network. Complete a bounded local transaction. Restore connectivity and watch the transaction move from local storage to reconciliation.

## Security section

### Headline
Offline does not mean unverified.

### Body
The prototype uses device-bound credentials, signed transaction envelopes, counters, replay controls, local ledger integrity, and backend reconciliation checks to constrain what can be accepted during degraded connectivity.

## Research section

### Headline
Built to be measured, attacked, and questioned.

### Body
ResilientPay is evaluated through controlled simulations and device experiments covering unreliable transport, duplicate messages, delayed synchronization, replay attempts, reconciliation conflicts, and risk-model performance.

## Closing section

### Headline
A payment should remain understandable even when the network does not.

### Action
READ THE PROTOCOL
