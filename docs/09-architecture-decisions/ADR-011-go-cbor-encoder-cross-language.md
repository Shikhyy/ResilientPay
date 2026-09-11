<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/09-architecture-decisions/ADR-011-go-cbor-encoder-cross-language.md
Authority: Architecture Decision Record
This file is normative unless explicitly marked as informative in its body.
-->
# ADR-011 - Cross-Language CBOR Invariant

> **Document role:** Architecture decision record.

## Status
Accepted

## Decision
The Go backend canonical CBOR encoder MUST produce byte-for-byte identical output to the Rust SDK (ciborium) for identical protocol input. Any change to the encoding logic is a protocol-breaking change (Class B/C change).

## Rationale
The core backend reconciliation logic re-derives canonical bytes from submitted transaction fields to verify the Ed25519 signature. If the Go backend produces different CBOR bytes than what the Rust SDK on the client signed, the signature verification will fail and all transactions will be rejected. 

## Enforcement mechanism
The test `TestCrossSDKTestVector_CBORMatches` in the Go backend (`internal/crypto/crypto_test.go`) MUST never be skipped or modified to relax its strict byte-for-byte check against the frozen Gate 3 vector.

## Scope
Any modification to:
- CBOR field order (13-element array)
- UUID encoding (must be raw 16-byte `bstr`, not hyphenated string)
- Integer width formatting (must use shortest form)
- The signing domain separator (`b"resilientpay:payment-envelope:v1:"`)

...requires concurrently updating all 3 encoders (Rust, Go, Python) and re-pinning all 3 test files simultaneously.

## Libraries
The canonical encoders are implemented using:
- **Rust**: `ciborium`
- **Go**: `fxamacker/cbor/v2` (configured with `SortNone` and `IndefLengthForbidden`)
- **Python**: `cbor2` (default serialization)

## Frozen test vector hex values
This is the standard Gate 3 frozen vector.

**Input:**
```
tx_id         = 00000000-0000-0000-0000-000000000001
credential_id = 00000000-0000-0000-0000-000000000002
payer_key_id  = 00000000-0000-0000-0000-000000000003
merchant_id   = 00000000-0000-0000-0000-000000000004
nonce         = [0x01; 16]
amount        = 150 paise, currency = "INR"
counter       = 1
created_at    = 1_700_000_000
expires_at    = 1_700_003_600
protocol      = 1
seed          = [0x42; 32]
```

**Expected outputs (DO NOT CHANGE):**
```
CBOR bytes (106B):
  8d015000000000000000000000000000000001500000000000000000000000000000000250
  000000000000000000000000000000035000000000000000000000000000000004189663
  494e520150010101010101010101010101010101011a6553f1001a6553ff10f6f6

Public key (32B):  2152f8d19b791d24453242e15f2eab6cb7cffa7b6a5ed30097960e069881db12

Signature (64B):
  04bbdb205dd371101fb2ad439d5f1d38cfc9146f7edd157a22f3476c42b0112359c46f1f0
  ea16aa1300cff1576628264bcb2f33b1d4ac2d6d8f592e16406ee07
```

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
