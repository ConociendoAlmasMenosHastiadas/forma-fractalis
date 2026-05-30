# Fractal Rendering Capability Table

Tracks per-fractal support across rendering backends. Update this file whenever a
fractal gains or loses support for a backend.

Last updated: v0.2.9 (Wallpaper)

## Legend

- yes  — Implemented and tested
- no   — Not implemented
- n/a  — Not applicable (e.g., fractal structurally requires CPU-side logic)
- TBD  — Planned for a future release

## Table

| Fractal              | CPU f64 | CPU Hi-Prec            | GPU f32 | Notes                                                                 |
|----------------------|---------|------------------------|---------|-----------------------------------------------------------------------|
| Mandelbrot           | yes     | yes                    | yes     | Hi-prec supports all power values via BigFloat polar form; PT backend (power=2 only) added in v0.2.7 |
| Julia                | yes     | yes                    | yes     | Power parameter (k) added in v0.2.3; hi-prec (all power values) added in v0.2.3  |
| Burning Ship         | yes     | yes (v0.2.4)           | yes     | GPU shader added in v0.2.2; hi-prec added in v0.2.4                  |
| Sin Julia            | yes     | yes (v0.2.5)           | yes     | GPU shader added previously; hi-prec added in v0.2.5                 |
| Sinh Julia           | yes     | yes (v0.2.8)           | yes (v0.2.8) | Release fractal for v0.2.8; component-wise abs(sinh(z)^4) + c |
| Zubieta              | yes     | no                     | yes     | Hi-prec planned (future plan)                                         |
| Marek Dragon         | yes     | yes (v0.2.8)           | yes (v0.2.6) | GPU shader added in v0.2.6; hi-prec added in v0.2.8               |
| Lemon                | yes     | no                     | yes (v0.2.8) | GPU shader added in v0.2.8; hi-prec planned (v0.3.0)             |
| Cactus               | yes     | yes (v0.2.7)           | yes (v0.2.5) | GPU shader added in v0.2.5; hi-prec added in v0.2.7               |
| Tetration            | yes     | yes (v0.2.9)           | yes (v0.2.7) | GPU shader added in v0.2.7; hi-prec added in v0.2.9 via BigFloat complex log/exp |
| Tippets Mandelbrot   | yes     | no                     | yes     | GPU shader added in v0.2.3; hi-prec planned (future plans)            |
| Multifractal Julia   | yes     | no                     | yes (v0.2.4) | GPU shader added in v0.2.4; hi-prec planned (future plans)        |
| Insideout Dragon     | yes     | yes                    | yes     | Unhidden in v0.2.2; hi-prec f/g perturbation computed at f64          |
| Multi-Julia IFS      | yes     | yes (v0.2.4)           | yes (v0.2.4) | New in v0.2.4; orbit accumulation; hi-prec + GPU orbit added in v0.2.4 |
| ChaosSymmetry1       | yes     | yes (f64 orbit + BigFloat screen mapping) | yes (GPU f32, v0.2.5) | Orbit accumulation; hi-prec maps screen coords at high precision; GPU repurposes IFS param slots |
| Wallpaper            | yes     | yes (v0.2.9)           | yes (v0.2.9) | Release fractal for v0.2.9; orbit accumulation with per-screen seeds; hi-prec keeps seed/orbit arithmetic in BigFloat |
| Lace Julia           | yes     | yes (v0.2.6)           | yes (v0.2.6) | New in v0.2.6; rational map with poles; c parameter + escape radius  |

## Future Distribution Plan

The CPU hi-precision pipeline (`iterate_hiprec`) is implemented for Mandelbrot, Insideout Dragon,
and Julia (v0.2.3). Distribution to remaining fractals follows one per release:

| Release | Fractal           |
|---------|-------------------|
| v0.2.4  | Burning Ship      | (complete)
| v0.2.5  | Sin Julia         |
| v0.2.6  | Zubieta           |
| v0.2.7  | Cactus            |
| v0.2.8  | Marek Dragon      |
| v0.2.9  | Tetration         |
| v0.3.0  | Lemon             |
| v0.3.1  | Tippets Mandelbrot|
| v0.3.2  | Multifractal Julia|

When adding GPU support to a fractal:
1. Create the WGSL kernel file and register in `wgpu_backend.rs` (see AGENTS.md section 6c).
2. Add the fractal to the GPU test list in `core/src/gpu_test.rs`.
3. Update the GPU column in this table.

When adding hi-prec support to a fractal:
1. Implement `supports_hiprec() -> bool` (return true) and `iterate_hiprec(...)` in the fractal file.
2. Add unit tests (see AGENTS.md "Add Hi-Precision CPU Support" section for the testing checklist).
3. Update the row in this table.
