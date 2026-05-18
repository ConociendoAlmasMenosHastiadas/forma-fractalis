---
name: fractal-implementation
description: 'Use when adding a new fractal, integrating a new fractal parameter, or wiring a fractal through CPU, GPU, hi-prec, GUI, metadata, and tests.'
argument-hint: 'fractal name or parameter'
---

# Fractal Implementation

## When to Use
- Adding a new fractal type
- Adding a rendering-affecting parameter to an existing fractal
- Completing backend coverage for an existing fractal

## Guardrails
- A new fractal is not complete until CPU f64, CPU hi-precision, and GPU f32 paths all honor the behavior.
- A rendering-affecting parameter is not complete until it is wired consistently across all active backends.
- Metadata round-trip checks and capability tracking are part of the implementation, not optional cleanup.

## Procedure
1. Review the [full checklist](./references/checklist.md) before editing.
2. Implement or update the fractal in `core/src/fractals/` first.
3. Register the type, defaults, and metadata wiring before touching GUI controls.
4. Add or update GUI/input state in `gui/`, and set `needs_redraw` for every rendering-affecting change.
5. Extend hi-precision and GPU backends in the same slice of work.
6. Update `plans/capability_table.md` when backend support changes.
7. Run `cargo check --workspace`, targeted tests, and GPU parity tests when shader behavior changes.