<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/12_TRANSACTION_STATE_MACHINE.md
Authority: Protocol specification
This file is normative unless explicitly marked as informative in its body.
-->
# 12 - State Machine Implementation Contract

> **Document role:** Normative source-of-truth document.

## Domain model

Represent transaction states as a closed set. State mutation occurs through a single domain transition function or equivalent aggregate boundary. UI callbacks, transport listeners, and database mappers cannot directly assign final states.

## Event examples

`UserIntent`, `PolicyAccepted`, `SignatureCreated`, `TransportDelivered`, `EvidenceReceived`, `EvidenceVerified`, `LocalPersisted`, `SyncQueued`, `BackendReconciled`, `ConflictDetected`, and failure/cancellation events.

## Atomicity

Whenever a transition consumes offline budget or counter authority, the state/evidence update must be crash-safe. Use a transaction or durable event mechanism appropriate to the local store.

## Recovery

On app restart, derive state from durable evidence rather than assuming the last in-memory state completed.

## Testing

Generate allowed and forbidden transition tests from the normative state machine. Include crash/restart tests around every state that represents consumed authority.
