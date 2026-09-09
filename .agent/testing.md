<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: .agent/testing.md
Authority: Agent operating procedure
This file is normative unless explicitly marked as informative in its body.
-->
# Agent Testing Contract

> **Document role:** Coding-agent operating procedure.

## Required test levels

Every change must include the lowest appropriate test and all affected higher-level tests.

### Protocol/crypto

Mutation, replay, serialization, boundary, negative tests.

### Ledger/state

Legal/illegal transition tests, crash/recovery tests, idempotency.

### Transport

Loss, duplicate, reorder, timeout, malformed payload.

### Backend

Authentication, authorization, transaction idempotency, database atomicity.

### Mobile

Keystore behavior, restart/recovery, offline operation, device changes where supported.

## Test evidence

Agent reports must identify exact commands run and any tests not run, with reasons.

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
