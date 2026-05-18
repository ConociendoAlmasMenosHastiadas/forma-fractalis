---
name: gpu-shader-development
description: 'Use when adding or modifying WGSL kernels, shader composition, orbit shaders, GPU parameter mapping, or WGPU backend registration.'
argument-hint: 'fractal name or shader task'
---

# GPU Shader Development

## When to Use
- Adding GPU support for a fractal
- Editing WGSL kernels or shared shader code
- Registering new pipelines in the WGPU backend

## Procedure
1. Review the [GPU workflow reference](./references/workflow.md) before editing WGSL.
2. Keep shared logic in the appropriate common shader and place fractal-specific logic in a focused kernel file.
3. Map fractal parameters through the shared GPU parameter slots and keep them aligned with the CPU interpretation.
4. Register the shader or orbit shader in `core/src/gpu/wgpu_backend.rs`.
5. Run GPU parity validation after any WGSL, shader-registration, or coordinate-mapping change.