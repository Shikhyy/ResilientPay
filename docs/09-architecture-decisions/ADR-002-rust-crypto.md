<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/adr/ADR-002-rust-crypto.md
Authority: Architecture Decision Record
This file is normative unless explicitly marked as informative in its body.
-->
# ADR-002 - Rust Security-Critical Core

> **Document role:** Architecture decision record.

## Status
Accepted for prototype architecture

## Decision
Isolate serialization/cryptographic protocol-critical logic in a Rust module with a narrow FFI/API boundary.

## Rationale
Rust offers memory-safety properties and explicit error handling suitable for a security-critical core. It does not remove the need for cryptographic review.

## Consequence
Cross-language build/test complexity increases. Keep the interface small and heavily tested.

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
## Boundary design

Rust should expose a narrow, versioned interface for canonicalization/parsing/verification rather than becoming a second application architecture. The mobile private-key operation remains outside Rust when Android secure storage owns the key. The safest default is therefore: Kotlin validates user/domain inputs; Rust/certified libraries provide deterministic protocol parsing and verification; Android Keystore performs the private-key operation; Kotlin feeds the resulting signature back into the protocol engine.

## FFI rules

FFI messages must have bounded lengths, explicit encodings, and no borrowed pointers whose lifetime is unclear. Convert Rust errors into a stable error taxonomy at the boundary. Never expose internal cryptographic library errors directly to users.

## Research value

The Rust boundary should be justified by measurable safety/consistency benefits, not language preference. Maintain cross-language golden vectors to prove that Kotlin and Rust agree on canonical bytes and verification outcomes.
