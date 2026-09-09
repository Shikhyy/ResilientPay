<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/SYSTEM_ARCHITECTURE.md
Authority: Architecture specification
This file is normative unless explicitly marked as informative in its body.
-->
# System Architecture

> **Document role:** Normative source-of-truth document.

## Purpose
Define component boundaries so transport, security, state, persistence, and UI concerns remain separable.

## Logical architecture

```text
Payer Android                  Merchant Android
┌───────────────┐             ┌───────────────┐
│ UI            │             │ UI            │
│ Domain/Engine │             │ Domain/Engine │
│ Crypto Adapter│             │ Crypto Adapter│
│ Local Ledger  │             │ Local Ledger  │
│ Transports    │             │ Transports    │
└──────┬────────┘             └──────┬────────┘
       │     NFC / BLE / QR          │
       └────────────┬────────────────┘
                    │
              SMS / Internet
                    │
          ┌─────────▼─────────┐
          │ Backend API       │
          │ Reconciliation    │
          │ Credential/Test CA│
          │ Risk Adapter      │
          └─────────┬─────────┘
                    │
               PostgreSQL
```

## Responsibilities

### UI
Presents state and collects user intent. It must not authorize transactions or handle private keys directly.

### Payment Engine
Owns protocol orchestration, policy checks, state transitions, and coordination between ledger/crypto/transport adapters.

### Crypto Adapter/Core
Provides approved signing/verification primitives and canonical serialization. It does not decide business policy.

### Local Ledger
Persists local events and recovery state. It does not declare remote settlement finality.

### Transport
Moves protocol messages. It may fail; business semantics must remain transport-independent.

### Backend Reconciliation
Validates evidence, enforces backend policy, detects contradictions, and derives the prototype's reconciliation state.

### Risk Engine
Provides advisory risk assessment. It cannot bypass hard protocol controls.

## Architectural invariants

1. UI cannot call transport internals to perform an unauthorized payment.
2. Transport cannot mutate payment state except through the payment engine contract.
3. Crypto code cannot access UI state.
4. Backend recomputes/validates critical facts from submitted evidence.
5. No component silently assumes Internet availability.

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

## Failure isolation

A transport outage should affect a transport result, not rewrite domain rules. A database outage should produce a durable failure/pending decision, not an optimistic success. A risk-service outage should activate the approved deterministic fallback. A credential-service outage should not cause a client to invent credential validity.

## Scaling direction

The research prototype should optimize for correctness and observability before distribution. Introduce queues, caches, Kafka-like event buses, sharding, or multi-region complexity only when an experiment or measured bottleneck requires it. Every distributed component adds another failure mode that must then be modeled.
