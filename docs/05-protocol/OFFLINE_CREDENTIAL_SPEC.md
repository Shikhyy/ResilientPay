<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/08_OFFLINE_CREDENTIAL_SPEC.md
Authority: Protocol specification
This file is normative unless explicitly marked as informative in its body.
-->
# 08 - Offline Credential Specification

> **Document role:** Normative source-of-truth document.

## 1. Purpose

An offline credential is a bounded authorization artifact associated with a device/key and policy. It is not a bank credential and must not contain a UPI PIN.

## 2. Credential contents

Conceptual fields:

- `credential_id`
- `subject_key_id`
- `issuer_id`
- `issued_at`
- `expires_at`
- `policy_version`
- `max_value_per_tx` (prototype policy)
- `max_value_outstanding` (prototype policy)
- `max_counter` or equivalent bound
- status metadata
- issuer signature

The exact production meaning of these fields would depend on the regulated deployment.

## 3. Credential lifecycle

```text
REQUESTED → ISSUED → ACTIVE → SUSPENDED/REVOKED → EXPIRED
```

Transitions must be authenticated and auditable.

## 4. Provisioning

Prototype provisioning is simulated. The backend test authority signs or registers credentials using test keys. Production provisioning is explicitly out of scope.

## 5. Device binding

Bind the credential to a device-held key where practical. The application should use Android Keystore-backed keys and should not persist raw private keys in ordinary application storage.

## 6. Device change/loss

A lost-device workflow must invalidate or disable the old credential where supported by the architecture. The new device must not silently inherit the old local balance/authorization state.

## 7. Offline exhaustion

When the local policy budget is exhausted, the device must refuse additional offline authorization and require the appropriate online/provisioning path.

## 8. Separation of concerns

Credential validity is distinct from transaction validity. A valid credential does not automatically make every transaction valid.

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
