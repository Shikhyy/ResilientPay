
# Contributing

## Development model

ResilientPay uses a monorepo with short-lived feature branches.

`main` is always expected to represent an integrated, reviewable state.

Examples:

- `feat/sdk-payment-envelope`
- `feat/sdk-state-machine`
- `feat/backend-reconciliation`
- `feat/payer-offline-payment`
- `feat/transport-nfc`
- `feat/web-landing-page`
- `design/payer-offline-flow`

Do not create long-lived branches such as `sdk-dev`, `android-dev`, or `website-dev`. They create divergence and make integration harder.

## Work unit

Every change begins with an issue or task that identifies:
- problem
- requirement or design reference
- scope
- constraints
- acceptance criteria
- expected tests

## Pull requests

A pull request must explain:
- what changed
- why
- affected requirement IDs
- relevant protocol and architecture documents
- security implications
- tests executed
- visual verification for UI work
- known limitations
- breaking changes

Keep unrelated concerns separate.

## Conventional commits

Use messages such as:

`feat(sdk): add immutable payment envelope`

`test(protocol): cover invalid state transitions`

`fix(reconciliation): make duplicate submission idempotent`

`docs(protocol): clarify credential expiration semantics`

`security(crypto): reject malformed signature inputs`

Commits should be logical checkpoints, not a dump of an entire day's work.

## Review standard

A change must be reviewable from:
1. specification
2. implementation
3. tests
4. evidence

No claim should rely only on "the code appears to work."

## Agent-generated changes

Agents may create commits, but final merge authority remains human-controlled for architectural, protocol, security, research, and production-boundary decisions.
