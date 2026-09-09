<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/16_DATA_MODEL.md
Authority: Development contract
This file is normative unless explicitly marked as informative in its body.
-->
# 16 - Data Model

> **Document role:** Normative source-of-truth document.

## 1. Core entities

### Device

Represents a prototype device identity and key reference.

### Credential

Represents bounded authorization metadata and lifecycle state.

### Transaction

Represents the logical payment event and current derived state.

### TransactionEvent

Append-only evidence of a state transition or observation.

### ReconciliationBatch

Groups submitted transactions for retryable ingestion.

### RiskAssessment

Stores model version, score, and decision metadata.

### AuditEvent

Security and operational evidence, with strict redaction.

## 2. Core invariants

- `transaction_id` is unique.
- Credential counters are checked according to the protocol's counter domain.
- Reconciled transactions cannot revert to a pre-reconciliation state.
- Audit records are append-only.
- Monetary values are integer minor units.

## 3. Storage policy

Sensitive values must be encrypted/protected according to their classification. Private keys belong in platform/secure key storage, not in relational tables.

## 4. Prototype schema evolution

Schema changes require a migration, version bump, test coverage, and documentation update. Agents must not silently mutate production-like schema behavior just to pass tests.

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
