<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/14_RECONCILIATION_SPEC.md
Authority: Protocol specification
This file is normative unless explicitly marked as informative in its body.
-->
# 14 - Reconciliation Specification

> **Document role:** Normative source-of-truth document.

## 1. Purpose

Reconciliation converts disconnected local evidence into a consistent backend record for the synthetic environment.

## 2. Principles

- authenticate every submitted transaction;
- process duplicate submissions idempotently;
- never trust client-reported finality;
- record conflicts instead of hiding them;
- preserve immutable evidence events;
- make retry safe.

## 3. Ingestion flow

```text
Receive batch
   ↓
Parse + schema validate
   ↓
Authenticate signature
   ↓
Check credential status
   ↓
Check transaction ID
   ↓
Check counter / policy
   ↓
Compare prior evidence
   ├── exact duplicate → idempotent success
   ├── compatible new evidence → accept
   └── contradiction → conflict
   ↓
Persist event + derived state
   ↓
Return result
```

## 4. Idempotency

Repeated submission of the exact same valid transaction must not create multiple economic effects in the synthetic ledger.

## 5. Conflict handling

Conflicts are first-class records. The backend must preserve both evidence items and the rule that classified them.

## 6. Ordering

Do not assume arrival order equals event order. The protocol's IDs, counters, timestamps, and dependency references define order; transport arrival order is merely observation order.

## 7. Reconciliation result categories

- `ACCEPTED`
- `ALREADY_KNOWN`
- `REJECTED`
- `CONFLICTED`
- `PENDING_MANUAL_REVIEW` (optional research mode)

## 8. Research measurement

Record reconciliation latency from first local authorization to backend disposition, not just HTTP latency.

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
