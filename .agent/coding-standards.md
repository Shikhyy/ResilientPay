<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: .agent/coding-standards.md
Authority: Agent operating procedure
This file is normative unless explicitly marked as informative in its body.
-->
# Coding Standards

> **Document role:** Coding-agent operating procedure.

## Kotlin / Android

- Kotlin idioms and null-safety.
- Coroutines/Flow for asynchronous work.
- Jetpack Compose for UI.
- Room for relational local persistence.
- Android Keystore for key material.
- ViewModel/domain separation.
- No cryptographic code in UI classes.

## Rust

- `rustfmt` clean.
- `clippy` clean unless an exception is documented.
- Avoid `unsafe`; any required use needs a written justification and tests.
- Strong error types.
- Deterministic serialization.
- No panics for attacker-controlled protocol input.

## Go

- `gofmt`.
- Context propagation.
- Explicit error handling.
- Validate external inputs at boundaries.
- SQL statements parameterized.

## Python

- Type hints.
- Ruff/formatter as configured.
- pytest.
- Deterministic seeds for experiments.
- Separate data preparation from training and evaluation.

## General

Small functions, explicit contracts, meaningful names, no dead code, no unexplained constants, no magic numbers for policy.

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
