
# ResilientPay

ResilientPay is a research-grade prototype for connectivity-resilient payments. The project investigates how bounded offline authorization, local device-to-device exchange, store-and-forward communication, and eventual reconciliation can be combined while preserving transaction integrity.

This repository is intentionally organized as a professional product monorepo. The implementation is separated into core SDK technology, Android products, backend services, simulation, website, design, shared packages, tests, and infrastructure.

## Product boundary

ResilientPay is not intended to replace UPI, operate an unauthorized payment system, or represent a prototype transaction as final settlement. The prototype uses controlled test identities and simulated value unless an appropriately authorized production partnership and regulatory pathway is established.

## Repository map

- `docs/` contains research, requirements, architecture, security, protocol, development, experimentation, and operations specifications.
- `design/` contains the UX, visual, website, payer, merchant, component, accessibility, and visual QA source of truth.
- `sdk/` contains the reusable ResilientPay implementation foundations.
- `apps/` contains the payer and merchant Android products.
- `backend/` contains reconciliation and supporting server-side services.
- `simulator/` contains deterministic failure, attack, transport, and experiment simulation.
- `website/` contains the public product and research experience.
- `packages/` contains shared protocol types, design tokens, and test vectors.
- `tests/` contains cross-component verification.
- `infrastructure/` contains local development, CI, and future deployment definitions.
- `.agent/` contains coding-agent operating procedures.

## Core engineering sequence

Specification → threat model → protocol → state machine → SDK → ledger → reconciliation → simulator → Android → transports → risk research → experiments → public demo.

Do not reverse this order merely because a later component is more visible.

## Development philosophy

The project follows small, reviewable changes. Every significant implementation step should be linked to requirements, specifications, tests, and a conventional commit. Architectural and protocol decisions remain controlled decisions rather than implementation conveniences.
