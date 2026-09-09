
# SDK Release Process

## Purpose

The SDK is a product surface with consumers, compatibility obligations, and release evidence.

## Release stages

```text
development
→ internal validation
→ release candidate
→ compatibility review
→ security review
→ tagged release
→ changelog
```

## Versioning

Track:
- SDK version
- protocol version
- test-vector version where relevant

A programming API compatibility check is not sufficient when protocol semantics change.

## Release checklist

- unit tests pass
- contract tests pass
- security tests pass
- public API reviewed
- dependency review complete
- release notes prepared
- known limitations documented
- protocol compatibility checked
- test vectors versioned
- artifact reproducibility checked

## Research stage

Pre-1.0 releases should not imply production stability. Experimental protocol changes may be intentionally incompatible, but those changes must be explicit and documented.
