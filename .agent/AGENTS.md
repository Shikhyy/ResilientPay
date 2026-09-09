<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: .agent/AGENTS.md
Authority: Agent operating procedure
This file is normative unless explicitly marked as informative in its body.
-->
# AGENTS.md - Coding Agent Constitution

> **Document role:** Coding-agent operating procedure.

## Read this first

Before changing code, read:

1. `.agent/AGENTS.md`
2. `.agent/workflow.md`
3. `.agent/security-rules.md`
4. `.agent/DEFINITION_OF_DONE.md`
5. the relevant architecture/development specifications named by the task.

## Project status

This is a **research prototype** for a connectivity-resilient payment protocol. Do not represent prototype behavior as live UPI capability or production settlement.

## Architecture rules

- The protocol is transport-independent.
- Supported transport adapters: Internet, NFC, BLE, QR, SMS.
- Transport code cannot contain payment authorization logic.
- UI cannot directly access private cryptographic keys.
- Payment/domain logic owns transaction state.
- Cryptographic primitives are isolated behind a dedicated crypto boundary; Rust is the default security-core language.
- Android apps use Kotlin/native Android.
- Backend uses Go + PostgreSQL.
- ML uses Python.

## Security rules

- Never invent cryptography.
- Never store private keys in plaintext.
- Never send UPI PINs, passwords, or private keys via SMS.
- Never log secrets.
- Never use floating point for money.
- Never trust client-reported final states.
- Never use timestamps alone for replay prevention.
- Never disable TLS/security checks to “make the prototype work” in production paths.
- Never allow ML to override cryptographic or deterministic authorization.

## Change control

Do not silently change protocol fields, state transitions, credential semantics, counter rules, or trust boundaries. Propose the change, update the specification/ADR, and update tests before treating the change as complete.

## Coding-agent authority

The agent may implement a defined task. The agent may suggest improvements. The agent may not unilaterally redefine the protocol or regulatory assumptions.

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

## Agent decision ladder

When a task is ambiguous, the agent must first check the existing source-of-truth documents, then existing implementation, then tests. If ambiguity remains in a security/protocol matter, the agent must stop at the smallest decision boundary and report the alternatives instead of choosing silently. For ordinary implementation details that do not change external behavior, the agent may choose the simplest maintainable option.

## Required task report

End each task with a compact report: implementation summary; specification references; files changed; tests run; security checks; schema/API impact; docs/ADR impact; known limitations; unresolved decisions. This report is part of the development evidence loop.
