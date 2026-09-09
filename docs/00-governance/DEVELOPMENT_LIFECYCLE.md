
# Development Lifecycle

## Standard lifecycle

```text
Research question
→ requirement
→ threat/trust analysis
→ protocol or product design
→ architecture
→ task
→ implementation
→ tests
→ security/design verification
→ documentation sync
→ commit
→ pull request
→ CI
→ human review
→ merge
→ evidence/release
```

## Milestone levels

### M0 Repository control
Repository structure, agent rules, CI, branch protection, documentation index.

### M1 Protocol foundation
Payment envelope, amount model, credential abstraction, state machine, invariants.

### M2 Security foundation
Signing, verification, canonical serialization, counters, replay protection, credential lifecycle.

### M3 Ledger and reconciliation
Local event ledger, sync protocol, idempotency, conflict handling, authoritative backend.

### M4 Simulator
Deterministic payer, merchant, backend, transport and failure simulation.

### M5 Android integration
Android adapter, payer app, merchant app.

### M6 Physical transports
QR, NFC, BLE, SMS.

### M7 Risk research
Risk features, baseline model, evaluation, on-device inference where justified.

### M8 Research validation
Fault injection, security attacks, formal model checking, experiments, paper evidence.

### M9 Public product presentation
Landing page, live demo, documentation presentation, legal pages.

## Loop rule

Every milestone is itself a loop:
plan → implement → verify → review → commit → integrate.

No milestone is considered complete because code exists. It is complete when the corresponding evidence exists.
