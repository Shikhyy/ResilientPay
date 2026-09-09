<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/09_PAYMENT_PROTOCOL.md
Authority: Protocol specification
This file is normative unless explicitly marked as informative in its body.
-->
# 09 - Payment Protocol

> **Document role:** Normative source-of-truth document.

## Status
Authoritative research-prototype protocol specification.

## Goals
Transport-independent, cryptographically authenticated, replay-aware, deterministic, versioned, idempotent at reconciliation, and explicit about uncertainty/finality.

## Logical envelope

```text
PaymentEnvelope
├── protocol_version
├── tx_id
├── credential_id
├── payer_key_id
├── merchant_id
├── amount_minor
├── currency
├── counter
├── nonce
├── created_at
├── expires_at
├── previous_event_hash (optional by profile)
├── risk_class / policy_ref
└── signature
```

Only fields explicitly included by the schema are signed.

## Processing order

1. Parse/schema validation.
2. Protocol-version validation.
3. Signature verification over canonical bytes.
4. Credential status/validity check.
5. Counter/replay check.
6. Value/policy checks.
7. Payee/merchant checks defined by the profile.
8. Persist local evidence.
9. Expose the result to the user.

Transport success never equals payment authorization.

## Finality

Local authorization, receipt, storage, reconciliation, and settlement must be represented as distinct concepts. The prototype's `RECONCILED` state means the backend accepted the synthetic evidence according to the prototype policy; it does not represent real UPI settlement.

## Serialization

The signed representation must be deterministic. Canonical CBOR is the preferred research candidate. JSON may be used for human/debug/service representations only when signatures are verified against the canonical representation.

## Money

Use integer minor units; for INR, ₹1 = 100 paise. Never use floating point.

## Replay and duplicate handling

Use transaction IDs plus protocol-defined counters/credential state. Timestamps are supporting metadata, not the sole replay defense. Exact duplicate submissions must be idempotent at reconciliation.

## Versioning

Unsupported protocol versions are rejected. Protocol changes require an ADR, migration/compatibility analysis, and updated tests.

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
