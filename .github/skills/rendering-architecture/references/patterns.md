# Rendering Patterns

## Unified Rendering Pipeline
- `core/src/rendering_pipeline.rs` is the single source of truth for rendering behavior.
- Preview rendering should call `render_with_config()` instead of bypassing the pipeline with direct iteration helpers.
- Rendering changes should be made in the pipeline first so preview and export stay aligned.

## GUI Redraw Rules
- GUI state is grouped in `gui/src/app_state.rs`.
- Any control that changes rendered output must set `needs_redraw = true`.
- Missing redraw wiring is a user-visible bug because settings appear to do nothing.

## Fractal GUI Pattern
- Fractal-specific parameter controls follow the `FractalGUI` trait in `gui/src/fractal_gui.rs`.
- Prefer extending that pattern instead of adding ad hoc branching elsewhere.

## Coordinate Mapping and Polar Input
- GPU coordinate math must stay in sync with `FractalView::screen_to_complex()`.
- For polar complex-parameter UI, normalize `atan2()` output into `[0, 2pi]` so sliders do not snap unexpectedly.

## Orbit Accumulation
- Orbit-accumulating fractals route through `core/src/orbit_accumulation.rs` and the orbit-specific paths in `core/src/rendering_pipeline.rs`.
- Preserve deterministic sub-orbit seeding so output is stable across thread counts.
- Respect the memory threshold logic that switches between fold/reduce buffers and the shared atomic buffer.

## Failure Handling
- Do not hide rendering failures behind silent fallbacks.
- If a backend or precision mode cannot satisfy the request, surface the failure clearly.