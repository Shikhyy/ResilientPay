
# Agent Branching and Work Isolation

## Rule

Every coding agent task that changes code should occur on a dedicated short-lived branch unless the repository owner explicitly asks for a different workflow.

## Naming

Use:
`<type>/<scope>-<short-description>`

Examples:
- `feat/sdk-payment-envelope`
- `fix/ledger-recovery`
- `security/credential-validation`
- `test/reconciliation-conflicts`
- `design/payer-payment-state`

## Isolation

An agent must inspect the working tree before beginning and must not overwrite unrelated uncommitted work.

When multiple agents are active, use isolated worktrees where the chosen tooling supports them.

## Handoff

A handoff should include:
- branch
- latest commit
- changed files
- tests run
- known limitations
- unresolved decisions

## Conflict handling

Do not solve merge conflicts by choosing whichever implementation is newer without understanding the requirement. Re-check the specification, security implications, and ownership of the conflicting change.
