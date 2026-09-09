<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/11_TRANSPORT_SPEC.md
Authority: Protocol specification
This file is normative unless explicitly marked as informative in its body.
-->
# 11 - Transport Implementation Contract

> **Document role:** Normative source-of-truth document.

## Interface

All transports implement the same logical contract: report capabilities, accept a protocol message, deliver/receive bytes, and return transport outcomes. The adapter must not decide transaction validity or settlement.

## Internet

Use HTTPS with timeouts and retry-safe requests. A timeout after backend acceptance is a classic ambiguity and must be resolved through idempotent status/query behavior rather than blind resubmission.

## NFC

Treat proximity as a delivery mechanism. Test device discovery, handoff, interruption, payload size, and permissions. Never infer identity solely from physical proximity.

## BLE

Test permissions, discovery, connection loss, characteristic framing, concurrent peers, retries, and lifecycle cleanup. Connection success does not authorize payment.

## QR

Assume QR values can be copied or modified. The scanned payload must lead to authenticated merchant/protocol data before authorization.

## SMS

Treat SMS as telecom-assisted store-and-forward. Design for loss, duplication, delay, multipart fragmentation, filtering, and device/SIM changes.

## Adapter acceptance

Every adapter must pass common contract tests using the same protocol fixtures.

## Implementation contract

Transport implementations are adapters.

They receive a protocol object, serialize it according to the protocol contract, move it through the selected channel, and reconstruct the protocol object at the receiving boundary.

A transport must not:
- change signed fields
- re-price the transaction
- reinterpret credential semantics
- create settlement authority
- bypass replay checks

## Test strategy

Every transport adapter should have:
- serialization tests
- availability tests
- loss/retry tests
- duplicate-delivery tests
- malformed-message tests
- integration tests against the common protocol interface

A transport-specific failure must become a typed transport result, not an ambiguous payment rejection.
