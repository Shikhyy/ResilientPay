<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/research/25_TRACEABILITY.md
Authority: Requirements specification
This file is normative unless explicitly marked as informative in its body.
-->
# 25 - Requirements Traceability

> **Document role:** Normative source-of-truth document.

## 1. Purpose

Prevent implementation drift by linking research goals to requirements, threats, protocol invariants, tests, and evidence.

## 2. Traceability chain

```text
Research question
  → requirement
  → protocol rule
  → implementation module
  → test
  → experiment
  → paper claim
```

## 3. Example trace

| Research need | Requirement | Protocol rule | Test | Metric |
|---|---|---|---|---|
| Prevent replay | FR-003 / SEC | Counter checked | replay test | replay rejection rate |
| Survive message duplication | FR-006 | Idempotent tx ID | duplicate delivery test | incorrect duplicate effects |
| Transport independence | FR-005 | Same PaymentEnvelope | cross-transport integration | equivalent outcomes |
| Detect tampering | SEC | Signed canonical fields | mutation test | 100% mutation rejection |
| Measure offline risk | RESR | bounded policy | simulation | exposure/conflict rate |

## 4. Change rule

Any change to a protocol invariant requires updating:

- requirements;
- threat model, if affected;
- protocol/state machine;
- tests;
- ADR if it changes an accepted decision;
- experiment definition if it changes a research variable.

## 5. Review checklist

Before merging a significant change, confirm that the changed behavior is traceable to a requirement and that no requirement has become unimplemented or contradicted.

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
