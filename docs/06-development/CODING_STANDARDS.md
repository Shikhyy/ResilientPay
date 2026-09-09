<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/CODING_STANDARDS.md
Authority: Development contract
This file is normative unless explicitly marked as informative in its body.
-->
# Coding Standards

> **Document role:** Normative source-of-truth document.

## 1. General

Code must be readable, deterministic where required, explicit about failure, and easy to test. Prefer small modules, pure validation functions, immutable domain values, and adapters around side effects. Avoid clever abstractions that obscure protocol semantics.

## 2. Kotlin / Android

Use Kotlin, Jetpack Compose, Coroutines/Flow, Room, Android Keystore, BiometricPrompt where the approved authentication model calls for it, and WorkManager for durable background work. UI state is derived from domain/application state. Do not perform signing, counter mutation, or reconciliation decisions in composables.

## 3. Rust

Use `rustfmt`, `clippy`, explicit error types, safe Rust by default, bounded parsing, and maintained cryptographic libraries. `unsafe` requires a written justification and targeted tests. No direct implementation of cryptographic primitives.

## 4. Go

Use `gofmt`, `go vet`, context propagation, explicit timeouts, wrapped errors, and transactional database handling. Separate transport/API validation from reconciliation domain logic.

## 5. Python

Use type hints, `pytest`, reproducible virtual environments/lockfiles, deterministic seeds, and scripts that can regenerate research outputs. Keep exploratory notebooks separate from reusable evaluation code.

## 6. Money and IDs

No floating-point money. Use integer minor units and explicit currency. Never generate transaction identity from display text or timestamps alone.

## 7. Logging

Never log private keys, PINs, secrets, or complete sensitive payment payloads. Use error codes and correlation IDs.

## 8. Review

Every protocol-sensitive code review must compare the diff with the source-of-truth specification and relevant test vectors.

<!-- Deepened -->
## Naming and API style

Use domain names consistently: `transaction_id`, `credential_id`, `counter`, `amount_minor`, `protocol_version`, `message_id`. Avoid synonyms such as `paymentId` in one language and `transactionKey` in another unless the concepts are genuinely different. This is especially important across Kotlin/Rust/Go/Python boundaries.

## Error semantics

Use typed errors/results at security boundaries. Callers should be forced to handle invalid signatures, unsupported versions, persistence failures, and transport failures explicitly. Avoid boolean methods whose `false` value hides whether the cause was malformed input, unavailable infrastructure, or a security rejection.
