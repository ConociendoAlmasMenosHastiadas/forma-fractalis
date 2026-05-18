---
name: rendering-architecture
description: 'Use when modifying preview or export rendering, redraw behavior, orbit accumulation, render backend selection, or coordinate mapping parity.'
argument-hint: 'rendering change or subsystem'
---

# Rendering Architecture

## When to Use
- Changing preview or export rendering behavior
- Touching backend selection, redraw rules, or view math
- Working on orbit accumulation, perturbation, or parity-sensitive rendering

## Procedure
1. Start from `core/src/rendering_pipeline.rs`; both preview and export inherit behavior from that pipeline.
2. Review the [architecture notes](./references/patterns.md) before making parity-sensitive changes.
3. If a GUI control affects rendering, wire `needs_redraw` in the corresponding `gui/` state or UI path.
4. Keep CPU and GPU coordinate mapping aligned, including polar-angle normalization where relevant.
5. If the fractal uses orbit accumulation, preserve deterministic sub-orbit seeding and existing OOM safeguards.
6. Validate the smallest affected rendering path before widening scope.