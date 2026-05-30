# Hi-Precision Checklist

## Core API Expectations
- Import `astro_float::{BigFloat, RoundingMode}` directly.
- Override `supports_hiprec()` and `iterate_hiprec()` for escape-time fractals that support deep zoom.
- For orbit-accumulating fractals, implement the hi-precision orbit path instead of forcing the escape-time API to handle it.

## Implementation Notes
- Use the requested bit width and `RoundingMode::ToEven` consistently.
- Guard against `NaN` and `inf` after each iteration step.
- Use `cmp()` for BigFloat comparisons instead of assuming standard operators exist.

## Fallback Rules
- If only some parameter combinations are supported, call the standard f64 path explicitly and document the limitation.
- Do not silently pretend hi-precision succeeded when the code really fell back.

## Testing
- Cover an in-set point and an escaping point.
- Compare the hi-precision and f64 paths on boundary-free points.
- Smoke test across 64, 128, 256, 512, and 1024 bits.