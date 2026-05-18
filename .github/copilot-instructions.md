# Project Guidelines

## Scope
- Use this file as the single always-on Copilot instruction file for the repo.
- Keep detailed, task-specific procedures in `.github/skills/`.
- Most active code lives in the Cargo workspace members `core/` and `gui/`.

## Workflow
- Most work is driven by active release plans in `plans/v*.md`; if a task maps to a plan, update progress and implementation notes while you work.
- `plans/` should contain only unreleased work. Move completed plans to `plans/old_plans/` immediately, and prefix archived sub-plans with the release version.
- If a feature is deprecated in one release plan, schedule the actual removal or completion in a later plan instead of leaving the follow-up implicit.
- This project is heavily LLM-assisted. If you discover a repeated rule that belongs in always-on guidance or a dedicated skill, call it out.

## Documentation
- Never use emojis in documentation, plan files, README content, CHANGELOG entries, or release notes.
- Do not include test counts in CHANGELOG entries.
- Each release requires a new fractal. Since v0.2.0, each release also needs a showcase image in `img_resources/showcase/` plus matching `README.md` and `index.html` updates.
- Colormaps are maintained in the `scala-chromatica` repository, not here. Do not add per-release colormap tasks to plans.

## Workspace Map
- `core/`: fractals, rendering pipeline, export, orbit accumulation, perturbation, GPU backend, and GPU test helpers.
- `gui/`: eframe application, CLI glue, grouped GUI state, export helpers, and fractal parameter UI.
- `build_scripts/windows-build.ps1`: Windows packaging script for release staging.
- `plans/capability_table.md`: backend coverage tracking.

## Architecture
- Preserve preview/export parity through `core/src/rendering_pipeline.rs`; preview rendering should call `render_with_config()` rather than bypassing the pipeline.
- GUI changes that affect visual output must set `needs_redraw = true`.
- Keep fractal-specific controls on the `FractalGUI` pattern in `gui/src/fractal_gui.rs`.
- Keep GPU coordinate mapping in lockstep with `FractalView::screen_to_complex()`; shader drift is a rendering bug.
- Orbit-accumulating fractals belong on `core/src/orbit_accumulation.rs` and must preserve deterministic sub-orbit seeding plus memory safeguards.
- Avoid silent fallback behavior. Surface failures so the user can correct them.

## Build and Test
- Default validation: `cargo check --workspace` and `cargo test --lib`.
- After dependency updates, rerun both commands.
- After WGSL or GPU backend changes, run the GPU parity workflow before considering the work done.
- Use `build_scripts/windows-build.ps1` only for Windows release packaging.

## Skills
- `plan-workflow`: plan editing, plan archiving, deprecations, and showcase bookkeeping.
- `fractal-implementation`: new fractals, new fractal parameters, metadata wiring, and capability tracking.
- `rendering-architecture`: unified pipeline, redraw rules, orbit accumulation, and parity-sensitive changes.
- `gpu-shader-development`: WGSL kernels, composed shaders, and GPU backend registration.
- `hi-precision-rendering`: astro-float integration and deep-zoom precision work.
- `release-workflow`: final release prep, versioning, packaging, and GitHub release steps.
- `gpu-test-validation`: GPU/CPU parity testing and failure diagnosis.