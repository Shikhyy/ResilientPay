<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/adr/ADR-009-hash-chain-local-ledger.md
Authority: Architecture Decision Record
This file is normative unless explicitly marked as informative in its body.
-->
# ADR-009 - Hash-Linked Local Ledger

> **Document role:** Architecture decision record.

## Status
Accepted for research

## Decision
Use an append-only local event ledger with cryptographic hash linkage.

## Rationale
It provides tamper-evident sequencing without requiring distributed consensus.

## Limitation
Hash chaining does not by itself provide global double-spend prevention or transaction settlement.

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
## Chain construction

Each event hash should be computed from a canonical event representation plus the previous event hash and a domain-separation label. This creates tamper evidence if an attacker changes an event in the middle without recomputing all later links.

## Important limitation

If an attacker fully controls the local device and can rewrite the entire chain including its initial anchor, a local hash chain alone cannot prove history. Stronger evidence requires an external checkpoint, secure hardware, or later backend anchor. This distinction must be visible in research claims.

## Verification

On recovery/sync, verify chain linkage before trusting derived local history. Fuzz event parsing and test truncated chains, duplicated events, reordered events, and modified previous hashes.
