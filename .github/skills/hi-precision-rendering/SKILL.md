---
name: hi-precision-rendering
description: 'Use when adding or modifying astro-float BigFloat rendering, deep-zoom support, precision fallbacks, or hi-precision orbit accumulation.'
argument-hint: 'fractal name or precision task'
---

# Hi-Precision Rendering

## When to Use
- Adding deep-zoom support to a fractal
- Debugging precision-sensitive rendering behavior
- Extending hi-precision orbit accumulation or fallback logic

## Procedure
1. Review the [hi-precision checklist](./references/checklist.md) before editing.
2. Implement or update the hi-precision path in the fractal or orbit implementation, not in an unrelated caller.
3. Keep fallback behavior explicit when only some parameter combinations are supported.
4. Re-run agreement tests on boundary-free points and smoke tests across supported bit widths.