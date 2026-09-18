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

### E2 — Canonical CBOR vs Compact JSON Encoding Benchmark
- **Execution**: 10,000 iterations per payload complexity class (`minimal`, `standard`, `max_meta`).
- **Commit**: `698c0063349fb246fbc31526af951fbcb81b4d05`
- **Results**:
  | Payload Class | CBOR Size | JSON Size | Size Reduction | CBOR Parse Latency | JSON Parse Latency | SMS Segments (CBOR vs JSON) |
  |---------------|-----------|-----------|----------------|--------------------|--------------------|-----------------------------|
  | **Minimal**   | 106 B     | 281 B     | **62.3%**      | 2.00 µs            | 2.58 µs            | **2 vs 4** segments         |
  | **Standard**  | 108 B     | 284 B     | **62.0%**      | 1.46 µs            | 2.60 µs            | **2 vs 4** segments         |
  | **Max Meta**  | 146 B     | 370 B     | **60.5%**      | 1.67 µs            | 2.94 µs            | **3 vs 5** segments         |
- **Finding**: Canonical CBOR cuts over-the-air payload volume by >60% across all representative sizes, halving SMS segment requirements and reducing parsing latency by up to 43%.

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

### E5 — ML Risk Control & Deterministic Invariant Benchmark
- **Execution**: 1,000 transactions (800 honest, 200 adversarial across 4 fault/attack vectors).
- **Commit**: `698c0063349fb246fbc31526af951fbcb81b4d05`
- **Results**:
  | Metric | Measured Value | Requirement / Bound |
  |--------|----------------|---------------------|
  | **Deterministic False Acceptance** | **0** | MUST be 0 (hard security invariant) |
  | **Deterministic False Rejection**  | **0** | MUST be 0 |
  | **ML Hard Rule Overrides**         | **0** | MUST be 0 (ADR-010 compliance) |
  | **Advisory High-Risk Flags**       | **42** (4.2%) | Calibrated against air-gap velocity bursts |
  | **Avg Risk Scoring Latency**       | **96.7 µs** | < 1 ms target |
- **Finding**: The advisory ML boundary (ADR-010) successfully flags subtle air-gap burst attacks without overriding deterministic cryptographic rules or producing false rejections for legitimate users.

### E6 — Centralized Hash-Chain vs Distributed Blockchain Audit Log
- **Execution**: 1,000 canonical transactions processed through both audit architectures.
- **Commit**: `698c0063349fb246fbc31526af951fbcb81b4d05`
- **Results**:
  | Performance Metric | Hash-Chain Local Ledger (ADR-009) | Blockchain / Merkle Audit Log (ADR-007) | Architecture Comparison |
  |--------------------|-----------------------------------|------------------------------------------|-------------------------|
  | **Throughput** | **389,439.8 tx/s** | 121,383.2 tx/s | **3.2x advantage** for hash-chain |
  | **Append Latency (Mean)** | **2.43 µs** | 8.08 µs | **3.3x lower latency** |
  | **Audit Verification (1k tx)** | **1.23 ms** | 8.83 ms | **7.2x faster verification** |
  | **Tamper Detection Time** | **602.2 µs** | 4,347.7 µs | **7.2x faster localization** |
  | **Storage Overhead** | 427.7 B / tx | 359.2 B / tx | Hash-chain stores 32B pointer per tx |
- **Finding**: As decided in ADR-007 and ADR-009, a sequential hash-chained ledger provides deterministic, tamper-evident audit guarantees with 3.2x higher write throughput and 7.2x faster audit verification compared to a distributed block-based Merkle structure, without requiring distributed consensus overhead on mobile/edge endpoints.

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
