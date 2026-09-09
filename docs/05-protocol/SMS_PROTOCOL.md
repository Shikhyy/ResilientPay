<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/10_SMS_PROTOCOL.md
Authority: Protocol specification
This file is normative unless explicitly marked as informative in its body.
-->
# 10 - SMS Transport Protocol

> **Document role:** Normative source-of-truth document.

## 1. Role

SMS is a **store-and-forward transport/control-plane mechanism**, not a trust root and not “offline” according to RBI's formal definition because telecom connectivity is required.

## 2. Message types

- `CREDENTIAL_SYNC`
- `PAYMENT_SUBMISSION`
- `PAYMENT_ACK`
- `RECONCILIATION`
- `REVOCATION`
- `STATUS_REQUEST`

The logical payment envelope remains the same regardless of transport.

## 3. Requirements

SMS messages must be treated as:

- delayable;
- duplicable;
- reorderable;
- lossy;
- potentially misrouted;
- subject to handset/OS and carrier restrictions.

## 4. Security

Do not place UPI PINs, passwords, private keys, or bearer secrets in SMS.

Authenticate application-level content using cryptographic signatures and protocol identifiers. Encrypt payloads only when confidentiality is actually required and with standard, reviewed mechanisms.

## 5. Encoding experiment

Compare:

- canonical CBOR + binary-safe transport encoding;
- compact JSON;
- purpose-built compact text encoding.

Measure payload length, segmentation, transmission time, loss/retry behavior, and reconciliation delay.

## 6. Idempotency

Each message must carry a message identifier and/or reference a unique transaction ID. The backend must accept duplicate delivery without duplicating the underlying transaction effect.

## 7. Failure handling

| Condition | Action |
|---|---|
| Temporary send failure | retry with bounded backoff |
| Duplicate message | idempotently acknowledge/process |
| Invalid signature | reject; security event |
| Unknown transaction | store/reject according to state policy |
| Unsupported version | reject; no guessing |
| Reordered event | defer/apply according to state machine |
| Malformed payload | reject |

## 8. Prototype implementation note

Android SMS permissions and background behavior are constrained by the platform and store/policy rules. The prototype may therefore expose an injectable `SMSTransport` abstraction and use a controlled test gateway before attempting handset-level SMS integration.

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

## Important semantic boundary

SMS requires telecom connectivity. It is therefore treated in this project as a store-and-forward resilience transport, not as proof that the underlying payment was performed with no communication path.

## Delivery assumptions

The design must tolerate:
- delay
- loss
- duplicate delivery
- reordering
- truncation or malformed content
- wrong destination behavior where applicable

The security property comes from authenticated protocol data and backend validation, not from the transport channel.

## Operational consideration

Actual commercial SMS use may require platform permissions, approved gateways, policy compliance, and telecom integration. Prototype code should isolate these concerns behind an adapter so that testing does not depend on production SMS infrastructure.
