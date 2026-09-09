<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/security/07_CRYPTO_SPEC.md
Authority: Security specification
This file is normative unless explicitly marked as informative in its body.
-->
# 07 - Cryptography Specification

> **Document role:** Normative source-of-truth document.

## 1. Scope

This document specifies the cryptographic roles and rules. It intentionally avoids inventing cryptographic primitives.

## 2. Approved primitive policy

Use well-reviewed platform/library implementations of modern, appropriate algorithms. The exact algorithm is an ADR-level decision and must be versioned in the protocol. The default research candidate is Ed25519 for transaction signatures and SHA-256 for hashing where these are appropriate to the selected implementation stack.

Do not implement cryptographic algorithms manually.

## 3. Signature model

The payer/device produces:

```text
canonical_encode(PaymentEnvelopeCore)
        ↓
       sign(private_key)
        ↓
      signature
```

The verifier recomputes the exact canonical byte sequence before verification.

## 4. Signed fields

At minimum, the signed core should bind:

- protocol version;
- transaction ID;
- credential ID;
- payer device identifier or key reference;
- merchant identifier;
- amount in minor units;
- currency;
- counter/sequence value;
- nonce where required;
- issuance/expiry metadata where defined;
- previous ledger reference where used;
- policy/risk class where relevant.

## 5. Canonicalization

JSON object ordering must not be relied upon for signatures. The prototype should use a deterministic encoding such as canonical CBOR or another explicitly specified canonical binary format.

## 6. Hash chaining

A local ledger may use:

```text
H0 = domain-separated genesis value
Hn = Hash(domain || canonical_event || H(n-1))
```

This provides tamper-evident linkage. It does not, by itself, prove that an event is economically valid.

## 7. Replay protection

Use protocol identifiers and monotonic state such as credential ID + counter. Timestamps can support expiry, analytics, and ordering hints, but must not be the sole replay defense.

## 8. Key lifecycle

The design must specify:

- generation;
- registration/provisioning;
- storage;
- rotation;
- revocation;
- device loss/change;
- credential expiry;
- compromise recovery.

## 9. Crypto failure policy

Signature verification failure, malformed cryptographic input, or key-status failure is a security rejection, not a retryable transport error.

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
