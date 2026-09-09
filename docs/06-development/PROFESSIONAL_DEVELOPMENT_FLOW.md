
# Professional Development Flow

## Product workstream model

The project is a monorepo with independent workstreams that share a controlled protocol:

```text
Protocol
   ├── SDK
   ├── Android payer
   ├── Android merchant
   ├── Backend
   ├── Simulator
   └── Transports

Design
   ├── Website
   ├── Payer UX
   └── Merchant UX
```

## Feature lifecycle

1. Create issue.
2. Link requirement.
3. Identify affected protocol/domain state.
4. Identify architecture and design references.
5. Create short-lived branch.
6. Implement smallest coherent increment.
7. Add tests.
8. Run security checks.
9. Run visual/accessibility checks for UI.
10. Review diff.
11. Update documentation.
12. Create Conventional Commit.
13. Open PR.
14. Run CI.
15. Perform human review.
16. Merge.
17. Record release or experiment evidence where appropriate.

## Why this matters

The same user-visible feature may cross several systems. A professional process keeps the boundaries visible.

Example: "offline NFC payment" affects:
- protocol envelope
- credential policy
- state machine
- SDK
- NFC adapter
- payer UI
- merchant UI
- ledger
- reconciliation
- test vectors
- simulator
- visual QA

These changes should be linked but not mixed into an unreviewable implementation branch.

## Change-size rule

If implementation uncovers an unplanned architecture change, stop and split:
- approved implementation work
- architecture/protocol decision work

Do not bury the decision inside a feature branch.
