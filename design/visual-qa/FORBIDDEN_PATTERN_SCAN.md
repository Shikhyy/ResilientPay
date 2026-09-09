
# Forbidden Pattern Scan

This scan is required for visual releases.

## Reject the change when it introduces

- gradients as decorative treatment
- pure white primary surfaces
- drop shadows
- generic icon-library visual identity
- rainbow or neon palettes
- purple and black visual theme
- soft generic card radiuses
- glassmorphism
- bento-grid marketing structures
- exactly three feature cards as a default section structure
- fake testimonials
- fake metrics
- fake terminal windows
- sparkle or decorative floating effects
- animated arrows
- generic hover transitions
- emoji-based UI
- missing skeleton loading states
- missing Terms of Service
- missing Privacy Policy

## Review method

Search implementation and rendered output.

Check computed styles as well as source files because a component may inherit a forbidden pattern from a design library.

## Exceptions

An exception must be explicitly documented with:
- reason
- affected component
- reviewer
- expiry or re-evaluation point

No exception should be created merely because a library defaults to the prohibited treatment.
