<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/architecture/security/05_THREAT_MODEL.md
Authority: Security specification
This file is normative unless explicitly marked as informative in its body.
-->
# 05 - Threat Model

> **Document role:** Normative source-of-truth document.

## 1. Security objective

Protect transaction integrity, credential authenticity, replay resistance, controlled offline exposure, and reconciliation consistency under an adversary who may control transport conditions and may partially control a client device.

## 2. Assets

- Private signing keys.
- Offline credentials.
- Transaction counters.
- Local ledger entries.
- Transaction signatures.
- Synthetic account balances.
- Reconciliation records.
- Risk signals.
- Device identity metadata.

## 3. Trust boundaries

```text
[User/UI]
   |
   v
[Payment Domain] ---- [Crypto Boundary]
   |
   +---- [Transport Boundary]
   |
   +---- [Local Ledger]
   |
   v
[Backend API] ---- [Reconciliation DB]
```

No transport is trusted merely because it is proximity-based or operator-managed.

## 4. Adversaries

### A1 - Network attacker
Can observe, drop, delay, reorder, duplicate, or modify transport messages.

### A2 - Replay attacker
Reuses a valid signed transaction or valid credential material after its intended use.

### A3 - Compromised client
Can run modified code, inspect application storage, invoke APIs directly, or alter local presentation state.

### A4 - Malicious merchant/device
Attempts to submit malformed, duplicated, or contradictory transaction evidence.

### A5 - Insider/backend abuse
Attempts to modify records, bypass policy, or alter reconciliation outcomes.

## 5. Threat register

| ID | Threat | Control | Evidence |
|---|---|---|---|
| T-001 | Signature forgery | Modern digital signature | Crypto tests |
| T-002 | Message tampering | Sign canonical transaction | Mutation tests |
| T-003 | Replay | Credential ID + monotonic counter + idempotency | Replay tests |
| T-004 | Double spend | Bounded local authorization + reconciliation conflict model | Simulator/security tests |
| T-005 | Key extraction | Platform secure storage / non-exportable keys where available | Device security tests |
| T-006 | Message duplication | Idempotent transaction ID handling | Integration tests |
| T-007 | Message reordering | Explicit state/event model | State tests |
| T-008 | Clock manipulation | Counters/expiry rules not based solely on device time | Security tests |
| T-009 | Backend trust abuse | Server-side verification and authoritative state | API tests |
| T-010 | Sensitive logging | Redaction policy | Log tests/review |

## 6. Security invariants

1. A modified signed field must never verify.
2. An already-consumed credential counter must not authorize the same logical spend twice under the configured policy.
3. An invalid signature must not enter a reconciled state.
4. A terminal reconciled state must not transition backward.
5. Transport adapters cannot independently authorize payments.

## 7. Residual risk

Offline authorization cannot eliminate all economic risk while communication is absent. The objective is to **bound and measure** the risk, not to claim that offline double-spend is impossible in every architecture.

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
