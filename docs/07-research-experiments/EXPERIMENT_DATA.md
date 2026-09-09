<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/19_EXPERIMENT_DATA.md
Authority: Research/operations documentation
This file is normative unless explicitly marked as informative in its body.
-->
# Experiment Data Contract

> **Document role:** Normative source-of-truth document.

## Dataset classes

- fully synthetic transaction data;
- synthetic connectivity/failure traces;
- synthetic fraud strategies;
- optional device telemetry generated specifically for research.

## Reproducibility

Each dataset must have a version, generation script, schema version, seed (when stochastic), and provenance note.

## Leakage control

Do not train risk models on data that has already been used to tune thresholds and then report those same observations as unbiased evaluation.

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
