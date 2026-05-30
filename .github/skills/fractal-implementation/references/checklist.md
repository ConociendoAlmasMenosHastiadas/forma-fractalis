# Fractal Integration Checklist

## Core Fractal Implementation
- Create or update the fractal in `core/src/fractals/your_fractal.rs`.
- Implement the `Fractal` trait methods that define the fractal's behavior, defaults, and parameters.
- Add unit tests for an in-set point, an escaping point, and agreement on boundary-free points.
- Keep the implementation in the workspace crates only (`core/` and `gui/`). Do not add a parallel root-level `src/` path.

## Module Registration
- Add the module and re-export in `core/src/fractals/mod.rs`.
- Register the fractal in the canonical `FractalType` definitions and factory/reset paths used by the active crates.
- If the fractal changes save/load metadata, update `core/src/export.rs` parsing and round-trip coverage.
- Ensure the active release list stays correct: only ship-ready fractals belong in `FractalType::all()`.

## GUI and Input Wiring
- Add any new input fields and parsing helpers in `gui/src/app_state.rs`.
- Expose the parameters through the `FractalGUI` pattern in `gui/src/fractal_gui.rs` and the GUI wiring in `gui/src/app_state.rs` or neighboring UI helpers.
- Any parameter or view change that affects rendering must set `needs_redraw = true`.
- If the fractal is intended for release, keep the release plan updated while wiring the GUI rather than treating plan sync as a later cleanup pass.

## Hi-Precision Support
- Override `supports_hiprec()` and `iterate_hiprec()` for escape-time fractals that support deep zoom.
- If only some parameter combinations are supported, fall back explicitly and leave a clear note in the GUI or implementation.
- Add smoke coverage across the supported bit widths: 64, 128, 256, 512, and 1024.

## GPU Support
- Add or update the WGSL kernel under `core/src/gpu/shaders/`.
- Register the shader in `core/src/gpu/wgpu_backend.rs` and keep parameter mapping aligned with the CPU path.
- If the fractal uses orbit accumulation, follow the orbit shader path instead of the escape-time shader path.

## Metadata and Capability Tracking
- Update `core/src/export.rs` so the fractal name round-trips through parse/serialize paths.
- Add or update round-trip tests that cover `FractalType::all()` and any fractal-specific GUI state.
- Update `plans/capability_table.md` whenever hi-precision or GPU support changes.

## Validation
- Run `cargo check --workspace`.
- Run the relevant unit tests, especially metadata round-trip coverage in `core`.
- Run GPU parity validation if you changed WGSL, shader registration, or shared coordinate mapping.
- If the change is expected to preserve classic CPU performance, compare against `cargo bench -p forma-fractalis-core --bench fractal_bench` and update `BENCHMARKS.md` when the result matters to the release.