<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/API_CONTRACT.md
Authority: Development contract
This file is normative unless explicitly marked as informative in its body.
-->
# API Contract

> **Document role:** Normative source-of-truth document.

## Source of truth
The normative HTTP contract is `openapi.yaml` when that file exists in the repository. This document defines the rules around that contract.

## Rules

- JSON/HTTP is a service representation; the protocol signature is computed over the canonical protocol representation, not arbitrary JSON field order.
- All mutating operations require authentication and idempotency semantics where repeat delivery is possible.
- Client-provided transaction state is treated as an observation, not as authoritative finality.
- Errors use codes from `docs/development/ERRORS.md`.
- Breaking API changes require a versioning decision and migration plan.

## Prototype endpoints

```text
POST /v1/test-credentials
POST /v1/reconciliation/batches
POST /v1/reconciliation/transactions
GET  /v1/transactions/{tx_id}
GET  /v1/credentials/{credential_id}
GET  /v1/health
```

These names are internal prototype APIs and are not UPI/NPCI interfaces.

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

## Contract-first implementation

The OpenAPI/schema contract should be treated as generated engineering truth only after the logical protocol contract is approved. Server-side validation must not be weakened to accommodate a mobile client bug; fix the client or change the contract through change control.

## Contract evolution

Every incompatible change gets a new API version or explicitly defined migration path. Never change the interpretation of an existing field while keeping the same name because doing so makes historical evidence ambiguous.
