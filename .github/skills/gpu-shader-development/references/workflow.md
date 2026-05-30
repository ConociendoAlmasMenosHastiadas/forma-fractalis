# GPU Workflow

## Escape-Time Shader Composition
- `core/src/gpu/shaders/common.wgsl` provides the shared framework and contains the `// {{FRACTAL_KERNEL}}` marker.
- Each `*_kernel.wgsl` file should define exactly one function: `fn iterate_fractal(c: vec2<f32>) -> u32`.
- Compose shared and fractal-specific code through the backend instead of duplicating common helpers.

## Orbit Shader Composition
- Orbit accumulation uses the orbit shader template and the `// {{ORBIT_KERNEL}}` marker instead of the escape-time template.
- Orbit shaders use a 1D dispatch and write many density hits per invocation via atomics.

## Parameter Mapping
- Reuse the shared GPU parameter slots (`param_0`, `param_1`) and document which fractal parameter each slot represents.
- A new parameter is incomplete if the CPU path uses it but the GPU path ignores it.

## Backend Registration
- Register new shaders in `core/src/gpu/wgpu_backend.rs`.
- Escape-time kernels need the standard shader-loading path and a `pipeline_name_for()` entry.
- Orbit kernels need the orbit-specific loading path and the orbit pipeline mapping.

## Parity Rules
- Keep `pixel_to_complex()` aligned with the CPU view mapping.
- Treat coordinate-mapping drift as a correctness bug, not a quality issue.

## Validation
- Run the GPU parity workflow after WGSL edits, backend registration changes, or rendering pipeline changes that affect GPU dispatch.