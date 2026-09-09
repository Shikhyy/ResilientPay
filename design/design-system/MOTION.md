<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/MOTION.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Motion and Interaction

> **Document role:** Normative UX/visual specification.

Motion exists to communicate system state, not to make the page look active.

## Approved motion

### State transition
A transaction node moves from one state to the next.

### Synchronization
A local event moves along a Signal Line toward the backend.

### Loading
Structural skeleton transitions may use a restrained opacity change.

### Demo mode
The simulation can animate transport availability changes.

## Avoid

- animated arrows
- constant floating objects
- decorative background movement
- bouncing controls
- springy cards
- autoplaying marketing animations

## Reduced motion

All nonessential animation must be disabled under prefers-reduced-motion.

The semantic state must remain clear without motion.
