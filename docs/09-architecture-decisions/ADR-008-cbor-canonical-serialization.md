<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/adr/ADR-008-cbor-canonical-serialization.md
Authority: Architecture Decision Record
This file is normative unless explicitly marked as informative in its body.
-->
# ADR-008 - Canonical Binary Serialization

> **Document role:** Architecture decision record.

## Status
Accepted for research

## Decision
Evaluate canonical CBOR as the primary compact, signed wire representation.

## Rationale
The protocol needs deterministic encoding and low payload overhead. JSON can remain a human-friendly transport representation when signatures are computed over canonical binary data.

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
## Canonicalization obligations

The team must select and pin an exact canonical CBOR profile/library behavior. Tests must cover map ordering, integer representation, text encoding, omitted/default fields, duplicate keys if relevant to the parser, and unknown-field handling. The signer and verifier must operate on the identical byte sequence.

## Compatibility

A readable JSON debug representation may be generated from the parsed object, but it is not a substitute for the signed bytes. Logs should use hashes/selected fields rather than raw signed payloads.

## Research consequence

Measure encoded size and parse/encode latency on realistic transaction sizes. Do not assume CBOR is automatically smaller once signatures, framing, and transport overhead are included.
