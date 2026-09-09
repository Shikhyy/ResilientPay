<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/adr/ADR-004-postgresql.md
Authority: Architecture Decision Record
This file is normative unless explicitly marked as informative in its body.
-->
# ADR-004 - PostgreSQL

> **Document role:** Architecture decision record.

## Status
Accepted

## Decision
Use PostgreSQL as the durable reconciliation database.

## Rationale
The project needs transactional guarantees, constraints, indexes, and clear relational audit/event modeling.

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

<!-- Deepened -->
## Integrity strategy

Use database constraints as a second line of defense. For example, a unique transaction ID prevents accidental duplicate logical transactions even when application retries race. Foreign keys prevent orphan evidence. Check constraints can reject negative/overflowing money representations.

## Event and state strategy

A transaction table may hold derived state while `transaction_events` stores immutable evidence. When both must change atomically, use a single transaction. If asynchronous processing is introduced, use an outbox or equivalent pattern so an accepted DB event cannot disappear between database commit and message publication.

## Research consequence

PostgreSQL provides a clean baseline for comparing centralized reconciliation with more complex alternatives. The experiment should measure added latency/storage, not assume a blockchain is better or worse in the abstract.
