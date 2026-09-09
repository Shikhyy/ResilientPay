<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/13_DOUBLE_SPEND_MODEL.md
Authority: Security specification
This file is normative unless explicitly marked as informative in its body.
-->
# 13 - Double-Spend and Offline Risk Model

> **Document role:** Normative source-of-truth document.

## 1. Problem

Without timely communication, two parties may receive locally valid evidence based on overlapping authorization capacity. No software claim should imply that a disconnected system has magically obtained global consensus.

## 2. Risk controls

The prototype can reduce risk with:

- bounded offline value;
- finite credential budget;
- monotonic counters;
- device binding;
- short credential validity windows;
- merchant/device constraints;
- reconnect/reconciliation requirements;
- deterministic conflict detection;
- risk scoring for additional throttling.

## 3. Counter semantics

A counter must have a precisely defined owner and uniqueness rule. It must not be incremented by transport adapters.

## 4. Conflict classes

- duplicate `tx_id` with same content;
- duplicate `tx_id` with different content;
- same credential counter observed with different transactions;
- impossible state progression;
- total offline exposure above policy;
- revoked/expired credential used after invalidation.

## 5. Research metrics

Measure:

- conflicting transactions per 10k attempts;
- reconciled vs rejected ratio;
- time to conflict detection;
- economic exposure under attack assumptions;
- false positive/false negative rates for risk controls.

## 6. Research honesty

A protocol should report the conditions under which double-spend is prevented, detected, delayed, or merely bounded. Use precise terminology instead of “double-spend proof” unless formally established.

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
