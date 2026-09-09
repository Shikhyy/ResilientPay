<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/11_TRANSPORT_SPEC.md
Authority: Protocol specification
This file is normative unless explicitly marked as informative in its body.
-->
# 11 - Transport Specification

> **Document role:** Normative source-of-truth document.

## 1. Rule

Transports carry protocol messages. They do not decide whether a payment is valid.

## 2. Common interface

Conceptual contract:

```kotlin
interface PaymentTransport {
    fun isAvailable(): Boolean
    suspend fun send(message: ProtocolMessage): TransportResult
    fun receive(): Flow<ProtocolMessage>
}
```

## 3. Transport adapters

### Internet

Primary online path. Uses authenticated HTTPS APIs.

### NFC

Short-range proximity transport for local exchange where supported devices permit it.

### BLE

Short-range data exchange where supported. Pairing/discovery behavior must be considered part of the threat model.

### QR

A visual transport. Dynamic QR data is a carrier; the signed protocol payload provides integrity/authenticity. Static QR should not be treated as proof that a payee is currently online.

### SMS

Store-and-forward transport. Requires telecom availability.

## 4. Adapter responsibilities

Each adapter may handle:

- discovery;
- connection setup;
- framing/chunking;
- timeout;
- retry;
- transport-specific acknowledgements.

Each adapter must not handle:

- balance authorization;
- risk policy;
- counter advancement;
- settlement finality;
- cryptographic policy decisions.

## 5. Transport neutrality test

Given the same valid `PaymentEnvelope`, the payment engine must produce equivalent business outcomes regardless of transport, subject only to transport-specific availability/failure.

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
## Common adapter test suite

Every adapter should pass the same abstract tests: send a valid envelope, recover the exact logical message, simulate loss, simulate duplicate delivery, simulate reordering, simulate timeout, inject corruption, and cancel an in-progress transfer. The test harness should compare domain outcomes rather than adapter-specific callback details.

## Selection policy

A transport selector may rank transports using availability, payload size, user preference, and policy. It must not silently choose a weaker security policy to obtain a successful delivery. For example, failure to use NFC does not justify removing signature verification and falling back to unauthenticated QR.
