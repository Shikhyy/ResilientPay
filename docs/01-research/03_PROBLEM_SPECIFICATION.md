<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/research/03_PROBLEM_SPECIFICATION.md
Authority: Research specification
This file is normative unless explicitly marked as informative in its body.
-->
# 03 - Problem Specification

> **Document role:** Normative source-of-truth document.

## 1. Problem statement

Conventional real-time payment flows rely on timely communication with remote infrastructure. In weak-connectivity environments, a user may still have a legitimate intent to pay and a merchant may still need to accept payment, but the parties cannot immediately obtain centralized confirmation.

The protocol problem is therefore to support a **bounded local authorization event** while preserving integrity and enabling later reconciliation.

## 2. Connectivity model

```text
C0 - Full connectivity
Internet available → normal online path

C1 - Internet unavailable, telecom available
SMS/USSD or equivalent store-and-forward path may carry authenticated control/data messages

C2 - No Internet/telecom, local proximity available
NFC/BLE/QR can transfer protocol messages directly between nearby devices

C3 - Full isolation
No remote or local channel. The application may create/queue local state only if policy allows; it must not claim remote settlement.
```

The C1 path is not “offline” under the RBI definition because telecom connectivity is required. It is a resilience/store-and-forward mode.

## 3. Core problem dimensions

- Identity: which device/credential is acting?
- Authorization: is this transaction allowed locally?
- Integrity: was the transaction modified?
- Replay: was old authorization reused?
- Double spending: can the same offline capacity be consumed twice?
- Availability: can legitimate users transact under partial failure?
- Reconciliation: how are divergent local views converged?
- Auditability: can the system explain what occurred?

## 4. Threat-driven requirements

The design must assume a malicious user can:

- replay messages;
- duplicate or reorder messages;
- modify unsigned content;
- copy application files;
- attempt device migration;
- induce transport failures;
- submit the same signed envelope multiple times;
- manipulate local clocks;
- attempt to bypass UI-level checks.

## 5. Research success metric

The system is successful only if resilience does not come from silently weakening security. Every increase in offline availability must be expressed as an explicit, measurable increase in risk or state uncertainty.

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
