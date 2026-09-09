
# Branching and Releases

## Branches

`main` is the integration branch.

Use short-lived branches:
- `feat/...`
- `fix/...`
- `refactor/...`
- `security/...`
- `test/...`
- `docs/...`
- `design/...`
- `research/...`

## Branch scope

One branch should represent one coherent review unit.

A branch may span multiple components only when the change is intrinsically cross-cutting, such as a protocol version update.

## Release branches

Create release branches only for a stabilization period:
`release/sdk-v0.5.0`

After stabilization:
- run full verification
- review changelog
- confirm documentation
- tag the release
- merge or close according to repository policy

## Version axes

Track at least:
- SDK version
- application version
- protocol version
- backend API version

These must not be conflated.

## Rollback

Any release candidate must have a known rollback strategy for:
- application deployment
- backend deployment
- database migration
- SDK distribution
- protocol compatibility.

A protocol change that cannot be safely rolled back requires explicit architecture review.
