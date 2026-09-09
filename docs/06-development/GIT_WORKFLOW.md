
# Git Workflow

## Branch model

Use `main` as the integration branch. Feature branches are short-lived and scoped to a coherent task.

Examples:
- `feat/sdk-state-machine`
- `feat/backend-reconciliation`
- `feat/payer-offline-payment`
- `feat/transport-nfc`
- `feat/web-live-demo`
- `design/landing-page`

## Branch creation

Before creating a branch:
- inspect current working tree
- ensure base branch is current
- confirm task scope
- identify related documentation

## Commit model

Commit after major verified implementation checkpoints. Commits should tell a reviewer what became true at each point.

## Pull request model

A PR should be small enough for a reviewer to understand the change from the specification and diff. Include requirement IDs, test commands, security considerations, and screenshots for visual changes.

## Merge policy

Do not merge code that:
- fails required CI
- contains known critical security failures
- contradicts an approved specification
- introduces undocumented breaking changes

## History preservation

Do not rewrite shared history merely to create a cleaner narrative. The commit sequence is part of engineering provenance.
