<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/research/23_PAPER.md
Authority: Research specification
This file is normative unless explicitly marked as informative in its body.
-->
# 23 - Paper Framework

> **Document role:** Normative source-of-truth document.

## Candidate title

**Risk-Adaptive Multi-Modal Offline Payments: A Cryptographically Secure Store-and-Forward Architecture for Low-Connectivity UPI Environments**

## 1. Abstract structure

- Motivation: resilience under degraded connectivity.
- Existing context: offline payment mechanisms already exist.
- Gap: unified protocol semantics across heterogeneous transports and failure conditions.
- Method: signed transaction envelope, bounded offline credential, state machine, reconciliation, risk layer.
- Evaluation: transport, reconciliation, double-spend, ML, centralized vs blockchain audit.
- Result: quantify resilience/security trade-offs.
- Limitation: prototype and synthetic environment.

## 2. Contributions

1. Connectivity-state model.
2. Transport-independent transaction envelope.
3. Bounded offline authorization model.
4. Tamper-evident local ledger.
5. Conflict-aware reconciliation protocol.
6. Risk-adaptive policy experiment.
7. Formal threat and state analysis.
8. Empirical comparison across transports.

## 3. Related-work discipline

Explicitly cite and distinguish RBI/NPCI mechanisms such as UPI Lite and UPI Lite X from the proposed research contribution.

## 4. Evaluation requirements

Every quantitative claim must map to a reproducible experiment and configuration.

## 5. Limitations

State clearly that synthetic credentials, balances, and transactions are not production UPI settlement and that physical-device compromise can reduce guarantees.

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
