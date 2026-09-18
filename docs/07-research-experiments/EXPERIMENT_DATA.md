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

## Measured Research Experiment Results

### E1 — Transport Performance Benchmark
- **Execution**: Seed 42, 200 transactions per transport profile (1,000 txns total).
- **Commit**: `a6efa7a35cba464e5987024867a0f2500ac1076f`
- **Results**:
  | Transport | Success Rate | Loss Rate | Avg Latency | CBOR Payload | SMS Segments |
  |-----------|--------------|-----------|-------------|--------------|--------------|
  | **Internet** | 100.0% | 0.0% | 50.0 ms | 109.8 B | 1 (N/A) |
  | **NFC** | 98.5% | 1.5% | 19.7 ms | 109.8 B | 1 (N/A) |
  | **BLE** | 95.5% | 4.5% | 95.5 ms | 109.8 B | 1 (N/A) |
  | **QR** | 100.0% | 0.0% | 200.0 ms | 109.8 B | 1 (N/A) |
  | **SMS** | 93.0% | 7.0% | 2790.0 ms | 109.8 B | 1 segment (153B cap) |

### E3 — Reconciliation Robustness under Injected Faults
- **Execution**: 5 seeds (1..5) × 100 transactions per scenario (2,000 transactions total).
- **Results**:
  | Injected Fault Scenario | Convergence Rate | Conflict Rate | Rejection Rate | False Acceptance Count |
  |-------------------------|------------------|---------------|----------------|------------------------|
  | `duplicate_submission` | 100.0% | 0.0% | 0.0% | **0** (Idempotent 200 OK) |
  | `reordered_delivery` | 91.0% | 0.0% | 0.0% | **0** |
  | `delayed_delivery` | 100.0% | 0.0% | 0.0% | **0** |
  | `lost_message` | 0.0% | 0.0% | 0.0% | **0** |
- **Security Guarantee**: `false_acceptance_count == 0` across all 2,000 transactions.

### E4 — Double-Spend & Replay Resistance
- **Execution**: 50 trials per parameter combination (3 budgets: 100/500/2000 paise × 3 lifetimes: 5m/30m/1440m × 2 attacker strategies: `replay_same_counter`, `reuse_credential_after_budget_exhausted`).
- **Results**:
  - `false_acceptance_count`: **0** across all 900 simulated attack trials.
  - `economic_exposure_paise`: **0 paise** (no fraudulent value undetected at reconciliation).
  - `conflict_detection_time_txns`: **2.0 transactions** (exact second transaction with duplicate counter triggers immediate `CONFLICT` state).

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
