<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/17_API_SPEC.md
Authority: Development contract
This file is normative unless explicitly marked as informative in its body.
-->
# 17 - API Specification

> **Document role:** Normative source-of-truth document.

## 1. API purpose

The prototype backend exposes APIs for credential provisioning simulation, transaction reconciliation, status inspection, and research telemetry.

## 2. Representative endpoints

```text
POST /v1/test-credentials
POST /v1/reconciliation/batches
POST /v1/reconciliation/transactions
GET  /v1/transactions/{tx_id}
GET  /v1/credentials/{credential_id}
GET  /v1/health
```

These are prototype endpoints, not claims about any UPI or bank API.

## 3. Authentication

Use authenticated service-to-service requests. Prototype credentials must be synthetic and environment-specific.

## 4. Request validation

The backend must validate:

- schema;
- version;
- required fields;
- integer amounts;
- canonical transaction content;
- signature;
- credential status;
- counter rules;
- idempotency.

## 5. Response model

Every mutating endpoint should return a deterministic status/result code and a correlation identifier.

## 6. OpenAPI source of truth

The implementation should maintain `openapi.yaml` beside this document. Generated clients/servers must be treated as build outputs, not edited manually when the contract is generated.

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
