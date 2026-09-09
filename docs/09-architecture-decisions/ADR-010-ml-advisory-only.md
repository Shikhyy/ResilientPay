<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/adr/ADR-010-ml-advisory-only.md
Authority: Architecture Decision Record
This file is normative unless explicitly marked as informative in its body.
-->
# ADR-010 - ML Is Advisory Only

> **Document role:** Architecture decision record.

## Status
Accepted

## Decision
ML may estimate risk and influence policy within explicit bounds, but cannot override cryptographic, credential, counter, or hard-value checks.

## Rationale
Security authorization must remain deterministic and auditable. ML is uncertain and can fail or drift.

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
## Policy separation

The implementation should expose two separate functions: deterministic `validate_transaction()` and advisory `assess_risk()`. This prevents an accidental architecture where model output becomes part of cryptographic verification. The hard validator runs first or remains independently authoritative.

## Threshold governance

Thresholds must be versioned configuration, not hard-coded inside model code. Each threshold change should produce a measurable experiment and be recorded with the model version.

## Failure and drift

Model unavailability, degraded input quality, or detected drift must have a documented deterministic fallback. The fallback must be at least as conservative as the approved baseline; it must not become an allow-all path.
