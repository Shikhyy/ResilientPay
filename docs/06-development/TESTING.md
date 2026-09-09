<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/TESTING.md
Authority: Development contract
This file is normative unless explicitly marked as informative in its body.
-->
# Testing Contract

> **Document role:** Normative source-of-truth document.

The normative test strategy is in `docs/development/21_TEST_PLAN.md`. The coding-agent execution rules are in `.agent/testing.md`.

## Minimum quality loop

```text
Implement → Unit test → Boundary/integration test → Security test
          → Static analysis → Diff review → Traceability update
```

A passing happy-path test is insufficient for protocol work. Negative and adversarial cases are mandatory for cryptographic/state/transport boundaries.

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
## Contract-test fixtures

Maintain shared fixtures for: valid payment, invalid signature, changed amount, changed merchant, expired credential, counter replay, duplicate submission, unsupported version, and corrupted serialization. These fixtures should be language-neutral so multiple services can consume the same expected outcomes.

## Test ordering

Run cheap deterministic tests on every commit. Run integration/security suites on every PR. Run real-device transport tests on scheduled/lab jobs. Run large simulator experiments only when required and record their artifacts rather than making them blocking for every source change.
