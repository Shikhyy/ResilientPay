<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/research/26_DEVELOPMENT_GATES.md
Authority: Research specification
This file is normative unless explicitly marked as informative in its body.
-->
# 26 - Development Gates and Feedback Loop

> **Document role:** Normative source-of-truth document.

## Purpose

This document is the operating loop that keeps research intent, architecture, agent implementation, and evidence synchronized.

## Gate 0 - Repository bootstrap

Required: repository skeleton, agent rules, source register, project manifest, buildable empty services/apps.

Exit: an agent can read the repo rules and build the project without touching payment logic.

## Gate 1 - Research baseline

Required: charter, regulatory boundary, existing-system comparison, problem statement, requirements, threat model.

Exit: novelty claims are cautious, scope is explicit, and threat assumptions are testable.

## Gate 2 - Protocol freeze candidate

Required: trust model, crypto profile, credential model, canonical envelope, state machine, transport model, double-spend model, reconciliation semantics.

Exit: two engineers can independently describe the same transaction flow and arrive at the same state/evidence model.

## Gate 3 - Protocol test vectors

Create canonical examples and negative vectors before building radio integrations.

Exit: Kotlin/Rust/Go participants agree on canonical bytes, signature verification, field mutation behavior, and error semantics.

## Gate 4 - Deterministic core

Build simulator + domain core + local ledger + reconciliation against an in-memory or local backend.

Exit: the core passes unit/property/security tests without any real NFC/BLE/SMS dependencies.

## Gate 5 - Transport integration

Add one transport at a time. Each transport must pass the common contract suite.

Exit: switching transport changes delivery mechanics only, not transaction semantics.

## Gate 6 - Risk/experiments

Add advisory ML after deterministic security controls work. Pre-register experiment methodology where practical.

Exit: model results can be reproduced from versioned data/configuration and do not override hard controls.

## Gate 7 - Research evidence

Run fault, performance, security, and reliability experiments.

Exit: every important paper claim points to measured evidence and an explicit limitation.

## Continuous review loop

```text
Implement → Test → Threat review → Traceability → Human decision
                    ↑                       ↓
                    └── specification update ┘
```

If tests expose an architectural flaw, fix the specification first when the change is semantic. If the specification is sound and code is wrong, fix code without changing the contract. This distinction is the central control mechanism for the project.
