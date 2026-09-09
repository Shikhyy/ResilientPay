<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/12_TRANSACTION_STATE_MACHINE.md
Authority: Protocol specification
This file is normative unless explicitly marked as informative in its body.
-->
# 12 - Transaction State Machine

> **Document role:** Normative source-of-truth document.

## 1. Canonical states

```text
CREATED
  ↓
AUTHORIZED
  ↓
SIGNED
  ↓
TRANSFERRED
  ↓
RECEIVED
  ↓
STORED
  ↓
SYNC_PENDING
  ├────────→ REJECTED
  ├────────→ CONFLICTED
  └────────→ RECONCILED
```

Some implementations may collapse states for simplicity, but the meaning must remain explicit.

## 2. Transition rules

| From | To | Allowed | Condition |
|---|---|---:|---|
| CREATED | AUTHORIZED | yes | policy permits |
| AUTHORIZED | SIGNED | yes | signature succeeds |
| SIGNED | TRANSFERRED | yes | transport accepts handoff |
| TRANSFERRED | RECEIVED | yes | recipient parses message |
| RECEIVED | STORED | yes | local persistence succeeds |
| STORED | SYNC_PENDING | yes | remote reconciliation unavailable/not complete |
| SYNC_PENDING | RECONCILED | yes | backend accepts evidence |
| SYNC_PENDING | REJECTED | yes | backend or local rule rejects |
| SYNC_PENDING | CONFLICTED | yes | conflicting evidence |
| RECONCILED | CREATED | no | terminal |
| RECONCILED | SYNC_PENDING | no | terminal |
| REJECTED | RECONCILED | no | requires explicit new protocol event |

## 3. State ownership

- Device creates local pre-reconciliation states.
- Backend determines reconciliation outcome for the prototype.
- The client cannot force a backend transaction into `RECONCILED` merely by changing local storage.

## 4. Crash safety

A state transition must be persisted atomically with the event that justifies it, or the recovery behavior must be explicitly defined.

## 5. Formalization

The state machine is a candidate for TLA+ modeling. Safety invariants must be checked independently from application unit tests where practical.

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
## Reference transition table

| Current | Event | Result | Notes |
|---|---|---|---|
| `CREATED` | `VALIDATION_REQUESTED` | `VALIDATING` | input accepted for checks |
| `VALIDATING` | `POLICY_ACCEPTED` | `AUTHORIZED` | deterministic controls passed |
| `AUTHORIZED` | `SIGNATURE_CREATED` | `SIGNED` | authenticated evidence now exists |
| `SIGNED` | `TRANSPORT_DELIVERED` | `TRANSFERRED` | transport success only |
| `RECEIVED` | `EVIDENCE_VALIDATED` | `LOCALLY_VERIFIED` | signature/credential/counter/policy pass |
| `LOCALLY_VERIFIED` | `PERSISTED` | `LOCALLY_RECORDED` | durable evidence exists |
| `LOCALLY_RECORDED` | `SYNC_QUEUED` | `SYNC_PENDING` | awaiting backend |
| `SYNC_PENDING` | `BACKEND_RECONCILED` | `RECONCILED` | server evidence accepted |
| `SYNC_PENDING` | `BACKEND_CONFLICT` | `CONFLICT` | contradictory evidence |

The exact event vocabulary may evolve, but any evolution must preserve the security meaning of the states.

## Implementation requirements

The implementation must encode transitions as an explicit model rather than as a collection of independent booleans. A transaction should not simultaneously claim mutually incompatible states such as `RECONCILED` and `SYNC_PENDING`.

## Transition checks

For every transition, the implementation must define:
- permitted source state
- required event
- preconditions
- mutations
- emitted audit/event record
- destination state
- failure behavior

## Persistence rule

The state machine should operate from persisted domain state and recorded events, not only in-memory UI state. After process termination and restart, the same transaction must reconstruct to a valid state.

## Testing rule

Tests must cover every documented valid transition and a representative set of invalid transitions that could occur through malformed input, retries, race conditions, or stale state.

## UI rule

Payer and merchant UIs must map labels to domain state. A visual "success" treatment cannot be used for a transaction that remains pending reconciliation.
