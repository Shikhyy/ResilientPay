<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: .agent/CHANGE_CONTROL.md
Authority: Agent operating procedure
This file is normative unless explicitly marked as informative in its body.
-->
# Change Control and Review Loop

> **Document role:** Coding-agent operating procedure.

## Purpose
Prevent coding-agent velocity from outrunning protocol, security, research, or regulatory correctness.

## Change classes

### Class A - Local implementation
No protocol/API/state/security change. Agent may implement directly, with tests.

### Class B - Contract change
Changes an interface, schema, API, serialization, state transition, or error code. Requires specification update + tests + review.

### Class C - Security/protocol change
Changes trust assumptions, cryptographic inputs, credential semantics, replay controls, double-spend model, or settlement/reconciliation semantics. Requires an ADR and explicit human review before implementation.

### Class D - Regulatory/production scope change
Introduces real value, production payment integrations, regulated participants, customer credentials, or claims of live payment operation. Prototype work must stop for that area and a new regulatory/security assessment is required.

## Required evidence

For B/C/D changes, the agent report must identify affected requirements, invariants, threats, tests, and documents.

## Review loop

```text
Change request
   ↓
Classify A/B/C/D
   ↓
Read affected specs
   ↓
Assess impact on invariants/threats
   ↓
Update spec/ADR first for B/C/D
   ↓
Implement
   ↓
Tests + static/security checks
   ↓
Traceability check
   ↓
Human review
   ↓
Merge
```

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
