
# ResilientPay Agent Instructions

You are working on ResilientPay, a research-grade connectivity-resilient payment prototype. This file is the mandatory entry point for coding agents.

## Before coding

Read:
1. this file
2. `.agent/workflow.md`
3. `.agent/security-rules.md`
4. `.agent/DEFINITION_OF_DONE.md`
5. `.agent/CHANGE_CONTROL.md`
6. `.agent/commit-rules.md`
7. the relevant requirement
8. relevant protocol/security/architecture documentation
9. relevant design documentation for UI work
10. relevant ADRs

Inspect the existing code before changing it.

## Authority

Protocol, security, trust, regulatory scope, and research methodology are human-controlled decisions.

Do not silently invent behavior where specifications are ambiguous.

## Architecture

The core SDK is transport-independent. Android applications consume the SDK. Transport adapters move protocol objects and do not decide payment validity. Backend reconciliation independently validates client submissions.

## Development loop

Inspect → plan → implement → unit test → integration/security/visual verification → review diff → update documentation → commit → report.

For a large task, break it into logical checkpoints.

## Commits

Create periodic, meaningful Conventional Commits after each major verified implementation checkpoint. Do not accumulate unrelated changes into a single final commit.

After committing, verify:
- working tree
- diff
- test result
- changed files
- no secrets
- documentation consistency

## Stop conditions

Stop and request a decision when:
- protocol semantics are ambiguous
- security guarantees are unclear
- trust assumptions change
- a new cryptographic primitive is required
- an API breaking change is needed
- an architectural boundary must move
- implementation would contradict a normative specification

Do not turn uncertainty into code.

## UI

Use the `design/` source of truth. Do not invent generic visual patterns. Every meaningful UI surface requires loading, empty, error, offline, and accessibility behavior where applicable.

## Final report

Every completed task must state:
- implementation
- tests
- security
- design if relevant
- documentation
- commits
- remaining risks
