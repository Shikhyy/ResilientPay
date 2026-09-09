<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/21_TEST_PLAN.md
Authority: Development contract
This file is normative unless explicitly marked as informative in its body.
-->
# 21 - Test Plan

> **Document role:** Normative source-of-truth document.

## 1. Test pyramid

```text
                E2E
              /     \
        Integration  Security
             /          \
            Unit / Property
```

## 2. Unit tests

Cover:

- serialization;
- signature creation/verification;
- credential checks;
- counter logic;
- state transitions;
- amount calculations;
- idempotency helpers.

## 3. Integration tests

Cover:

- Android/backend reconciliation;
- database transactions;
- transport adapters;
- retry queues;
- model inference integration.

## 4. Security tests

Must include:

- signature mutation;
- replay;
- duplicate transaction;
- counter reuse;
- tampered credential;
- malformed serialization;
- log-redaction checks;
- unauthorized API access;
- invalid state transition attempts.

## 5. Property/invariant tests

Examples:

- serialize → parse preserves signed fields;
- duplicate reconciliation is idempotent;
- terminal states never revert;
- amount arithmetic remains exact.

## 6. Device tests

Test actual Android devices for NFC/BLE/Keystore behavior; do not assume emulator behavior is representative for hardware-specific security paths.

## 7. Release gate

No milestone is complete until all mandatory tests pass and `.agent/DEFINITION_OF_DONE.md` is satisfied.

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
