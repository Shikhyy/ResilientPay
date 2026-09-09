<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/22_RELEASE_PROCESS.md
Authority: Development contract
This file is normative unless explicitly marked as informative in its body.
-->
# Release and Research Milestone Process

> **Document role:** Normative source-of-truth document.

## Milestone types

- `M0` - documentation/protocol baseline
- `M1` - pure software simulation
- `M2` - local Android offline prototype
- `M3` - multi-transport prototype
- `M4` - integrated reconciliation + risk experiment
- `M5` - research evaluation candidate

## Release gate

Before each milestone:

1. Freeze the relevant protocol version.
2. Run mandatory tests.
3. Record known limitations.
4. Verify traceability.
5. Confirm no real-money path exists.
6. Produce reproducible build/configuration metadata.

## Rollback

Each milestone must identify how to roll back client/backend versions without silently corrupting protocol state.

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
