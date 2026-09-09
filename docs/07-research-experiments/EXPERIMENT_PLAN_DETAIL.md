<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/research/19_EXPERIMENT_PLAN.md
Authority: Research/operations documentation
This file is normative unless explicitly marked as informative in its body.
-->
# 19 - Experiment Plan

> **Document role:** Normative source-of-truth document.

## 1. Objective

Quantify whether the proposed architecture improves connectivity resilience without unacceptable security or reconciliation cost.

## 2. Experimental matrix

### E1 - Transport performance

Compare Internet, SMS, NFC, BLE, and QR.

Metrics: latency, payload size, success rate, retries, failure modes.

### E2 - Encoding

Compare canonical CBOR and compact JSON for representative transaction sizes.

Metrics: bytes, parse time, signature input size, SMS segmentation count.

### E3 - Reconciliation robustness

Inject duplicate, reordered, delayed, and lost messages.

Metrics: convergence time, incorrect terminal states, conflict rate.

### E4 - Double-spend simulation

Vary offline budget, credential lifetime, reconnect interval, and attacker strategy.

Metrics: economic exposure, conflict detection time, false acceptance.

### E5 - ML risk control

Compare deterministic-only policy with risk-assisted policy.

Metrics: false acceptance, false rejection, calibration, coverage, latency.

### E6 - Centralized vs blockchain audit experiment

Compare a PostgreSQL/hash-chain design with a blockchain-based append-only audit variant.

Metrics: throughput, end-to-end latency, storage growth, operational complexity, audit traceability.

## 3. Reproducibility

Each experiment must record:

- code commit;
- dataset/synthetic seed;
- protocol version;
- model version;
- hardware/software environment;
- parameter configuration;
- output artifacts.

## 4. Statistical discipline

Repeat experiments where randomness is involved. Report uncertainty and do not overfit conclusions to a single run.

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
