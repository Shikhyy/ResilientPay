<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/research/00_PROJECT_CHARTER.md
Authority: Research specification
This file is normative unless explicitly marked as informative in its body.
-->
# 00 - Project Charter

> **Document role:** Normative source-of-truth document.

## 1. Project identity

**Working name:** ResilientPay

**Project class:** Research prototype / experimental systems project

**Primary question:** Can a payment transaction remain cryptographically authentic, bounded in risk, replay-resistant, transport-independent, and eventually reconcilable when the payer and/or merchant temporarily lack reliable Internet connectivity?

## 2. Vision

Build a technology layer that can sit conceptually above or alongside existing regulated payment infrastructure and provide a **connectivity-resilient transaction protocol**, without redefining UPI settlement or claiming to replace UPI.

## 3. Research objectives

- Define a transport-independent payment envelope.
- Define secure device-bound offline authorization with explicit limits.
- Define an append-only local transaction ledger with tamper-evident linkage.
- Support multiple transport modes without duplicating payment business logic.
- Define authenticated reconciliation and conflict handling.
- Evaluate double-spend exposure, failure recovery, latency, payload size, and operational resilience.
- Investigate risk-adaptive controls using ML without delegating cryptographic authority to ML.
- Compare centralized reconciliation with an optional blockchain-based audit experiment.

## 4. Success criteria

The project succeeds as a research prototype when it can demonstrate, with reproducible tests:

1. A valid transaction can be created and verified offline within a defined risk envelope.
2. Modifying signed transaction content is detected.
3. Replay and counter reuse are detected according to the protocol model.
4. Transaction state transitions obey the formal state machine.
5. Duplicate/reordered/lost transport messages converge safely during reconciliation.
6. The same payment logic works through multiple transport adapters.
7. Security failures are fail-closed.
8. Experimental results are reproducible and traceable to explicit hypotheses.

## 5. Scope

### In scope

- Android payer and merchant prototype apps.
- Local encrypted application data and append-only ledger.
- NFC, BLE, QR, Internet, and SMS transport adapters.
- Cryptographic signing and verification.
- Credential provisioning and lifecycle simulation.
- Backend reconciliation API.
- Risk scoring experiments.
- Simulator and fault injection.
- Automated testing and security testing.
- Documentation and research evaluation.

### Out of scope

- Real customer money.
- Real UPI settlement.
- Access to confidential bank/NPCI systems.
- Circumvention of payment-system authorization.
- Production issuance of credentials.
- Custom cryptographic algorithms.
- Blockchain as the authoritative payment ledger.

## 6. Research-only financial model

All prototype balances are synthetic. A test transaction represents a protocol event, not a bank transfer. Any references to INR are representation examples only unless explicitly marked as simulated.

## 7. Design principles

- Security before convenience.
- Explicit trust assumptions.
- Bounded offline value.
- Deterministic state transitions.
- Transport independence.
- Eventual reconciliation, not false finality.
- No floating-point money.
- No secrets in logs.
- No security decisions based solely on timestamps or ML.
- Protocol changes require tests and documentation updates.

## 8. Decision gates

### Gate 1 - Problem and feasibility

Required: charter, regulatory landscape, existing systems, problem specification, requirements, threat model.

Exit criterion: the research question is specific, existing mechanisms are distinguished from the proposed contribution, and security assumptions are explicit.

### Gate 2 - Protocol correctness

Required: trust model, crypto specification, credential specification, payment protocol, transport model, state machine, double-spend model, reconciliation specification.

Exit criterion: a transaction can be described end-to-end without relying on unstated behavior.

### Gate 3 - Engineering readiness

Required: data model, API contract, database schema, module boundaries, errors, observability, coding standards, ADRs.

Exit criterion: coding agents can implement without making architectural decisions implicitly.

### Gate 4 - Evidence

Required: experiment plan, simulator, test plan, results, paper draft.

Exit criterion: claims are supported by measurements and limitations are stated.

## 9. Review loop

Every implementation cycle must answer:

- Which requirement changed?
- Which protocol invariant is affected?
- Which threat is affected?
- Which tests prove the invariant still holds?
- Which ADR or specification needs updating?

No code change is considered complete until the documentation and tests are synchronized.

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
