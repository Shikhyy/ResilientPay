
# Local Development Environment

## Goal

A new developer or coding agent should be able to reproduce the core development environment using documented setup steps and no proprietary production credentials.

## Initial requirements

- Git
- supported JDK for Android work
- Kotlin/Gradle toolchain
- Rust toolchain for security-critical core where adopted
- Go toolchain
- Python environment for research/ML
- PostgreSQL
- container tooling where used

## Data

Use synthetic identities and simulated monetary value.

Real bank credentials, real customer data, and production keys are prohibited.

## Environment configuration

Configuration should be validated at startup. Missing required development configuration should produce an actionable error instead of silently using insecure defaults.

## Reproducibility

Document:
- supported versions
- setup commands
- database initialization
- migrations
- test commands
- simulator commands
- Android emulator/device requirements

## Agent requirement

Agents should run the smallest relevant local validation before asking for human intervention. They should report exact failures instead of claiming the environment is broken without evidence.
