
# Commit Rules

## Purpose

Git history is part of the engineering evidence. It should reveal how the system evolved, make rollback safe, and let future agents identify the origin of behavior.

## Commit timing

Commit after each meaningful verified checkpoint:
- domain model
- validation layer
- state machine
- serialization
- crypto capability
- database migration
- API endpoint
- UI flow
- transport adapter
- test suite improvement
- documented decision

Do not commit each tiny edit. Do not wait until a multi-day feature is complete.

## Conventional Commit format

`type(scope): summary`

Examples:
- `feat(sdk): add payment envelope`
- `test(sdk): add canonical serialization vectors`
- `fix(ledger): preserve event ordering on restart`
- `security(crypto): reject invalid signature length`
- `docs(protocol): document replay invariant`

## Verification before commit

Run the relevant:
- formatter
- linter
- tests
- security checks
- build

Inspect the diff.

Confirm no unrelated files are staged.

Scan for secrets and sensitive test data.

## Checkpoint loop

Implement → verify → inspect diff → commit → re-run targeted verification if commit hooks require it → report.

## Never

Do not rewrite shared history merely to make commit history look cleaner. Do not squash away security or protocol evolution when the chronology is valuable for review.
