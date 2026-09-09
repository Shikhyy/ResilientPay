<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/research/02_EXISTING_SYSTEMS.md
Authority: Research specification
This file is normative unless explicitly marked as informative in its body.
-->
# 02 - Existing Systems and Research Gap

> **Document role:** Normative source-of-truth document.

## 1. Existing mechanisms relevant to the project

### UPI Lite

An on-device low-value payment capability using existing UPI ecosystem protocols. The user enables it online and uses a pre-funded balance for supported low-value transactions.

### UPI Lite X

An NFC-based offline payment mechanism associated with UPI Lite. It demonstrates that proximity device-to-device transfer can be integrated into the existing ecosystem.

### UPI Tap & Pay

An NFC interaction pattern for obtaining a payee identifier from a compatible merchant artefact/device. It is relevant as a transport/discovery precedent, but is not itself the proposed research contribution.

### UPI 123PAY

A feature-phone-oriented UPI route using mechanisms such as IVR, missed call, and proximity sound. It demonstrates the broader principle that the UPI ecosystem can expose multiple access modes.

### Generic store-and-forward systems

SMS and other intermittently connected transports can carry authenticated messages that are accepted, persisted, retried, and reconciled later. The project should treat these as transport infrastructure, not as the source of trust.

## 2. Research gap

A useful research gap is not “offline payments do not exist.” Instead investigate:

> How can a single, formally specified payment transaction model preserve security invariants across multiple dissimilar connectivity modes while tolerating delay, duplication, reordering, loss, and temporary isolation?

Additional research questions:

- Can offline risk be made adaptive without weakening deterministic protocol controls?
- How should reconciliation resolve conflicting or duplicated state observations?
- What trade-offs occur between payload compactness, transport reliability, and cryptographic evidence?
- Does a centralized append-only reconciliation model outperform a blockchain-based audit model for this use case?

## 3. Novelty policy

Every paper claim must distinguish:

- **Existing ecosystem capability** - what RBI/NPCI already support.
- **Engineering synthesis** - combining known mechanisms into a coherent implementation.
- **Research contribution** - a new protocol property, formal model, algorithm, empirical result, or comparative analysis.

## 4. Competitive differentiation

The project is differentiated by:

1. One logical payment envelope across several transports.
2. Explicit state-machine semantics.
3. Device-bound credentials and anti-replay counters.
4. Reconciliation as a first-class protocol, not an afterthought.
5. Formal fault and attack testing.
6. Risk-adaptive but cryptographically bounded offline policy.

## 5. What must not be claimed

Do not claim the project invented NFC offline payments, UPI Lite X, UPI Lite, or SMS-based store-and-forward in general.

<!-- Enriched: detailed implementation guidance -->


## Engineering interpretation and implementation notes

### Normative language
The words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are used intentionally. MUST/MUST NOT define requirements that an implementation cannot change without a specification/ADR update. SHOULD/SHOULD NOT define strong recommendations that may be overridden only with a documented reason.

### State and evidence rule
A user-visible success message, transport callback, database row, or ML result is not independently authoritative. The authoritative meaning of an event comes from the protocol and state machine governing it. Any implementation that shortcuts the defined verification path is a protocol defect even when it makes a demo appear more reliable.

### Failure-first implementation
For every happy-path step, design its failure path before coding it. Ask: what if the process dies immediately before/after this write? What if the message arrives twice? What if the bytes are modified? What if the device clock is wrong? What if the backend response is lost after acceptance? What if a credential is revoked while the device is disconnected? The answer must be represented in state, error semantics, or an explicit documented residual risk.

### Reproducibility
Security-sensitive and research-sensitive behavior must be deterministic where practical. Test vectors, configuration, protocol version, database schema version, model version, and simulator seed are part of the evidence. A screenshot is not sufficient evidence for a security property.

### Change-control trigger
A change to a field, state, key lifecycle, counter rule, offline budget, transport meaning, reconciliation rule, risk boundary, or trust assumption MUST be treated as a specification change and reviewed through `.agent/CHANGE_CONTROL.md`.
