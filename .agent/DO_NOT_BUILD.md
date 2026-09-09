<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: .agent/DO_NOT_BUILD.md
Authority: Agent operating procedure
This file is normative unless explicitly marked as informative in its body.
-->
# Explicit Non-Goals

> **Document role:** Coding-agent operating procedure.

Do not build any of the following without a new approved scope and regulatory/security review:

- real UPI settlement;
- real bank credentials or customer PIN handling;
- customer-money custody;
- custom cryptography;
- security bypasses for Android or backend authentication;
- blockchain as the payment-authority path;
- “guaranteed delivery” assumptions for SMS/NFC/BLE;
- automatic promotion of local confirmation to financial settlement;
- ML as the authorization authority;
- hidden background transfer of sensitive payment information.

The prototype is an experimental system, not a financial service.

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

## Why these exclusions matter

Each non-goal protects the research result from becoming misleading. Real funds create regulatory, accounting, dispute, and customer-harm obligations; production credentials create real security consequences; blockchain would introduce a different trust/consensus problem; custom crypto would make the research impossible to review responsibly; and a hidden offline budget bypass would undermine the entire double-spend analysis.

## Escalation rule

If a proposed feature violates this document but appears necessary, do not implement it first and justify it later. Raise a change proposal with the research objective, risk, regulatory impact, and a safer prototype alternative.
