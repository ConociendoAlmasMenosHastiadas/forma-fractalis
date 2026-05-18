---
name: gpu-test-validation
description: 'Use when running or diagnosing GPU parity tests after WGSL edits, WGPU backend changes, shared coordinate-mapping changes, or rendering pipeline changes.'
argument-hint: 'optional fractal name'
---

# GPU Test Validation

## When to Use
- After adding or modifying any WGSL kernel
- After changing `common.wgsl`, orbit shader templates, or GPU parameter mapping
- After editing `core/src/gpu/wgpu_backend.rs` or GPU dispatch paths in the rendering pipeline

## Commands
- Test all supported GPU fractals: `cargo run --release -p forma-fractalis -- gpu-test`
- Test a specific fractal: `cargo run --release -p forma-fractalis -- gpu-test --fractal "Mandelbrot"`

## What to Check
- Failures save images under `temp/gpu_test/`; successful runs clean that directory.
- GPU initialization failures usually indicate driver or backend availability issues for the current machine.
- Pixel mismatches beyond tolerance usually mean shader logic drift or coordinate-mapping divergence.
- CPU-side failures point back to the fractal implementation rather than the shader.

## Follow-up
- If parity fails, inspect the shared coordinate math first, then the fractal-specific kernel, then the backend registration path.
- If the GPU is unavailable for the current session, record the skip rather than pretending validation ran.