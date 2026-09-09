<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/DATABASE.md
Authority: Development contract
This file is normative unless explicitly marked as informative in its body.
-->
# Database Contract

> **Document role:** Normative source-of-truth document.

## Source of truth
Migration files are the executable database contract. `docs/development/18_DATABASE_SCHEMA.md` defines the conceptual schema and invariants.

## Core tables

- `devices`
- `credentials`
- `transactions`
- `transaction_events`
- `offline_budgets`
- `reconciliation_batches`
- `risk_assessments`
- `audit_events`

## Rules

- Use database constraints for invariants that can be expressed safely at the DB boundary.
- Keep transaction events append-only.
- Use transactions when evidence persistence and derived-state updates must be atomic.
- Use integer minor units for money.
- Never store private keys in PostgreSQL.

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

## Database as an enforcement layer

Application checks and database constraints are complementary. The application explains the business rule; the database prevents a class of races even if two workers make the same write concurrently. Unique keys, foreign keys, check constraints, and transactional boundaries should therefore be treated as part of the security/correctness model.
