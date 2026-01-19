# Plan v0.2.0 - GPU Acceleration

This will be a major release including GPU processing capabilities. As other sections are changed, notes on how to address the GPU integration should be expanded on here.

## Performance Investigation Context

**Current Performance Issue** (v0.1.4 development):
- User reports significant CPU stress during high-iteration, supersampled exports
- Previous versions (v0.1.2/v0.1.3) handled similar workloads without issue
- Benchmarks show individual renders meet targets, but extended operations (exports) show degradation
- Need to investigate:
  1. Memory allocation patterns during export (repeated large allocations?)
  2. Supersample filtering overhead (Box filter on large buffers)
  3. Thread contention/scheduling issues in rayon parallel rendering
  4. Cache thrashing from non-contiguous memory access patterns

**Why GPU Acceleration Matters:**
- CPU-bound fractal rendering is fundamentally limited by sequential iteration per pixel
- Supersampling multiplies pixel count (4x4 = 16x work, 8x8 = 64x work)
- High iteration counts (4096+) stress CPU thermal limits on sustained exports
- GPU can process thousands of pixels in parallel, each in independent SIMD lanes

---

## GPU Implementation Approaches

### Option 1: WGPU (Recommended)

**Crate:** `wgpu` (WebGPU implementation for Rust)

**Pros:**
- Cross-platform (Windows/Linux/macOS/Web)
- Modern API with good Rust integration
- Can target WGSL shaders (WebGPU Shading Language)
- Already used by `egui/eframe` ecosystem (some integration benefits)
- Good documentation and active community
- Compute shader support for non-graphics tasks

**Cons:**
- Larger dependency tree
- Async initialization complexity
- May be overkill for pure compute workloads

**Integration Strategy:**
1. Add compute shader pipeline alongside CPU rayon pipeline
2. User preference setting: "Rendering Backend: CPU/GPU/Auto"
3. Auto mode: Use GPU for high-res exports (>1920x1080), CPU for preview
4. Shader writes iteration counts to buffer, CPU does colormap lookup
5. Fallback to CPU if GPU initialization fails

**Dependencies:**
```toml
wgpu = "0.18"
pollster = "0.3"  # For blocking on async GPU operations
bytemuck = "1.14"  # For safe buffer casting
```

**Shader Architecture:**
```wgsl
// Compute shader: fractal_compute.wgsl
@group(0) @binding(0) var<storage, read_write> output: array<u32>;  // Iteration counts
@group(0) @binding(1) var<uniform> params: FractalParams;

struct FractalParams {
    center_x: f32,
    center_y: f32,
    zoom: f32,
    max_iter: u32,
    width: u32,
    height: u32,
    power: f32,
    // ... other fractal parameters
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_x = global_id.x;
    let pixel_y = global_id.y;
    
    if (pixel_x >= params.width || pixel_y >= params.height) {
        return;
    }
    
    // Convert pixel to complex plane coordinates
    let c = pixel_to_complex(pixel_x, pixel_y, &params);
    
    // Run fractal iteration (Mandelbrot/Julia/etc)
    let iter = iterate_fractal(c, &params);
    
    // Write result
    let idx = pixel_y * params.width + pixel_x;
    output[idx] = iter;
}
```

**Code Structure Changes:**
```
src/gpu/
    mod.rs           - GPU backend trait
    wgpu_backend.rs  - WGPU implementation
    shaders/
        mandelbrot.wgsl
        julia.wgsl
        burning_ship.wgsl
        tippets.wgsl
```

**Rendering Pipeline Modifications:**
- `rendering_pipeline.rs` gains new enum: `RenderBackend { Cpu, Gpu, Auto }`
- GPU path: 
  1. Upload view params to uniform buffer
  2. Dispatch compute shader (workgroups of 16x16 threads)
  3. Download iteration buffer from GPU
  4. Apply colormap on CPU (or upload colormap as texture for GPU lookup)
  5. Apply filtering/supersampling (can also be GPU compute shader)

**Challenges:**
- Complex number operations in WGSL (no native complex type)
- Period detection requires atomic operations or multiple passes
- Interior coloring needs special handling
- Shader compilation/caching strategy

---

### Option 2: OpenCL

**Crate:** `opencl3` or `ocl`

**Pros:**
- Mature compute API specifically designed for parallel computation
- Wide hardware support (NVIDIA, AMD, Intel)
- More explicit control over memory and execution
- Can query device capabilities (threads, memory, etc.)

**Cons:**
- Windows/Linux only (no macOS support as of Mojave)
- More verbose API compared to WGPU
- Requires OpenCL runtime installation
- Less "Rusty" API feel

**Use Case:**
- If targeting professional/scientific users with discrete GPUs
- If need fine-grained control over GPU memory and execution
- If WebGPU overhead is too high for sustained compute

---

### Option 3: CUDA (via `cudarc` or RustaCUDA)

**Pros:**
- Best performance on NVIDIA hardware
- Extensive CUDA library ecosystem (cuBLAS, etc.)
- Native support for complex numbers in CUDA math libraries

**Cons:**
- NVIDIA-only (excludes AMD/Intel GPU users)
- Requires CUDA toolkit installation
- Less portable than WGPU/OpenCL

**Use Case:**
- If targeting high-end workstation/scientific computing users
- If need absolute maximum performance on NVIDIA hardware

---

### Option 4: Vulkan Compute (via `vulkano` or `ash`)

**Pros:**
- Modern low-level API with excellent performance
- Cross-platform (Windows/Linux/Android)
- Direct control over GPU resources

**Cons:**
- Very complex API with steep learning curve
- More boilerplate than WGPU
- Manual synchronization and memory management

**Use Case:**
- If already familiar with Vulkan
- If need absolute control for optimization
- Not recommended unless GPU expertise available

---

## Recommended Implementation Plan

### Phase 1: Foundation (v0.2.0-alpha)
1. **Add WGPU dependency** and initialize GPU device
2. **Implement simple Mandelbrot compute shader** (no period, no interior color)
3. **Backend selection UI** in settings (CPU/GPU/Auto)
4. **Benchmark GPU vs CPU** at different resolutions
5. **Graceful fallback** to CPU if GPU unavailable

### Phase 2: Feature Parity (v0.2.0-beta)
1. **Port all fractal types** (Julia, Burning Ship, Tippets, Powerbrot) to shaders
2. **Period detection** on GPU (requires two-pass or atomic operations)
3. **Interior coloring** in shader
4. **Log scale smoothing** in shader
5. **Supersample filtering** on GPU (compute shader for Box/Lanczos filtering)

### Phase 3: Optimization (v0.2.0-rc)
1. **Shader optimization** (reduce register pressure, optimize branching)
2. **Persistent GPU buffers** (avoid repeated allocation)
3. **Async GPU dispatch** (don't block UI during export)
4. **Tile-based rendering** for very large exports (>8K)
5. **GPU colormap lookup** (upload gradient as 1D texture)

### Phase 4: Polish (v0.2.0)
1. **Device selection** (if multiple GPUs available)
2. **GPU memory monitoring** (warn on low VRAM)
3. **Benchmark mode** (measure GPU speedup vs CPU)
4. **Documentation** on GPU requirements and troubleshooting
5. **Shader hot-reload** for development

---

## Architecture Considerations

### Trait-Based Backend System

```rust
// src/rendering_pipeline.rs
pub enum RenderBackend {
    Cpu,
    Gpu,
    Auto,  // Choose based on resolution/complexity
}

pub trait FractalRenderer {
    fn render(
        &self,
        config: &RenderConfig,
        target: RenderTarget,
    ) -> Vec<u8>;
}

// src/rendering.rs (current CPU implementation)
pub struct CpuRenderer;
impl FractalRenderer for CpuRenderer { /* existing rayon code */ }

// src/gpu/wgpu_backend.rs (new GPU implementation)
pub struct WgpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipelines: HashMap<String, wgpu::ComputePipeline>,
    // ...
}
impl FractalRenderer for WgpuRenderer { /* GPU compute dispatch */ }
```

### Heuristics for Auto Backend Selection

```rust
fn select_backend(width: u32, height: u32, max_iter: u32, supersample: u32) -> RenderBackend {
    let total_pixels = (width * height * supersample * supersample) as u64;
    let complexity = total_pixels * max_iter as u64;
    
    // GPU worthwhile for:
    // - High resolution (>= 1080p)
    // - High iteration count (>= 2048)
    // - Supersampling (>= 2x)
    // - Large exports (>= 2 megapixels)
    
    if total_pixels >= 2_000_000 || max_iter >= 2048 || supersample >= 2 {
        RenderBackend::Gpu
    } else {
        RenderBackend::Cpu  // Preview at low res/iter stays CPU (lower latency)
    }
}
```

### Handling Colormap on GPU

**Option A: CPU Colormap Lookup** (simpler, recommended for v0.2.0)
- GPU outputs iteration counts only (u32 buffer)
- CPU reads buffer and applies colormap (existing code)
- Pro: Reuses existing colormap system, easy to implement
- Con: CPU bottleneck for large buffers (but GPU still does 99% of work)

**Option B: GPU Colormap Texture**
- Upload gradient as 1D texture (1024 or 4096 pixels wide)
- Shader samples texture with normalized iteration value
- Pro: Fully GPU-accelerated, no CPU bottleneck
- Con: Requires interpolation in shader, more complex

### Period Detection on GPU

**Challenge:** Period detection requires comparing z values across iterations, which is inherently sequential.

**Option A: Remove period detection for GPU path**
- Simplest solution, document as limitation
- Most users don't use period detection anyway

**Option B: Two-pass GPU algorithm**
1. First pass: Compute iteration counts (no period)
2. Second pass: For pixels at max_iter, re-run with period checking
3. Pro: Accurate, only costs performance for interior points
4. Con: Complex, requires shader recompilation

**Option C: Approximate period on GPU**
- Check for |z_n - z_{n-k}| < epsilon for k = period
- Store last k z values in thread-local memory
- Pro: Single-pass, reasonably accurate
- Con: Limited by thread-local memory size

---

## Testing Strategy

### Performance Benchmarks
```rust
// benches/gpu_bench.rs
#[bench]
fn gpu_vs_cpu_mandelbrot_hd_1024(b: &mut Bencher) {
    let cpu_time = bench_cpu_render(1920, 1080, 1024);
    let gpu_time = bench_gpu_render(1920, 1080, 1024);
    println!("GPU speedup: {:.2}x", cpu_time / gpu_time);
}
```

### Correctness Tests
- Compare CPU vs GPU output pixel-by-pixel
- Allow small epsilon for floating-point differences
- Test all fractal types at multiple zoom levels

### Fallback Tests
- Ensure graceful degradation when GPU unavailable
- Test with unsupported GPU hardware
- Test with GPU driver failures

---

## Migration Path for Users

1. **v0.2.0-alpha**: Opt-in GPU support (experimental)
2. **v0.2.0-beta**: GPU enabled by default for exports, CPU for preview
3. **v0.2.0-rc**: Auto backend selection with override in settings
4. **v0.2.0**: Full GPU support with comprehensive docs

### Settings UI
```
[Performance]
├─ Rendering Backend: ●CPU  ○GPU  ○Auto
├─ GPU Device: [NVIDIA GeForce RTX 3080]  [Change]
└─ Fallback to CPU if GPU fails: ☑
```

---

## Known Limitations and Tradeoffs

### GPU Limitations
- **No arbitrary precision:** GPUs use f32 (single precision), CPU can use f64
  - Deep zoom (>1e10) may lose precision on GPU
  - Solution: Switch to CPU automatically at deep zoom levels
  
- **No dynamic dispatch:** Fractal type must be known at shader compile time
  - Need separate shader pipeline per fractal type
  - Solution: Pre-compile all shaders, select at runtime

- **Memory limits:** Large exports may exceed GPU VRAM
  - 8K export with 8x8 supersample = 16GB rendered, then 2GB final
  - Solution: Tile-based rendering for very large exports

### CPU Advantages
- **Better precision:** f64 vs f32
- **Dynamic behavior:** Easy to add new fractals without shader recompilation
- **No driver dependencies:** Works everywhere
- **Better for preview:** Lower latency (<16ms), no GPU init overhead

### Recommended Strategy
- **GPU for:** Exports, high resolution, high iteration, supersampling
- **CPU for:** Preview, deep zoom, rapid parameter changes, initial view

---

## Dependencies and Compatibility

### Minimum Requirements
- **Windows:** DirectX 12 or Vulkan 1.1
- **Linux:** Vulkan 1.1 or OpenGL 4.3
- **macOS:** Metal (via WGPU's Metal backend)
- **GPU Memory:** 1GB minimum, 2GB recommended

### Rust Dependencies (WGPU path)
```toml
[dependencies]
wgpu = "0.18"          # WebGPU implementation
pollster = "0.3"       # Block on async GPU operations
bytemuck = "1.14"      # Safe buffer casting
encase = "0.6"         # Shader uniform structs

[dev-dependencies]
criterion = "0.5"      # GPU benchmarking
```

### Build Considerations
- No additional build tools required (WGPU handles shader compilation)
- Shader files embedded in binary via `include_str!`
- Optional: Pre-compile shaders at build time with `naga` for faster startup

---

## Future Enhancements (Beyond v0.2.0)

### v0.2.1+: Advanced GPU Features
- **Adaptive sampling:** Render low-detail areas at lower resolution
- **Progressive refinement:** Show low-res result immediately, refine over time
- **Real-time pan/zoom:** GPU updates at 60fps during interaction

### v0.3.0: Multi-GPU Support
- Distribute tiles across multiple GPUs for very large exports
- Load balancing between integrated + discrete GPUs

### v0.4.0: Custom Shader System
- User-provided WGSL shaders for custom fractals
- Shader editor in GUI (stretch goal)

---

## Open Questions for Implementation

1. **Should period detection be supported on GPU initially?**
   - Recommendation: No, add in v0.2.1 if users request it

2. **Should colormap be applied on GPU or CPU?**
   - Recommendation: CPU for v0.2.0 (simpler), GPU for v0.2.1 (optimization)

3. **How to handle shader compilation errors?**
   - Recommendation: Fail gracefully with detailed error message, fallback to CPU

4. **Should GPU be required or optional dependency?**
   - Recommendation: Optional, use feature flag `gpu` (enabled by default)

5. **How to distribute GPU shaders with binary?**
   - Recommendation: Embed in binary via `include_str!`, no external files

---

## Performance Goals (v0.2.0)

### Target Speedups (GPU vs CPU)
- **HD (1920x1080) @ 1024 iter:** 5-10x faster
- **4K (3840x2160) @ 2048 iter:** 10-20x faster
- **8K (7680x4320) @ 4096 iter:** 20-40x faster
- **Supersample 4x4:** 10-15x faster (parallelizes filtering too)

### Export Time Targets
- **HD @ 2048 iter, 4x4 SS:** <5 seconds (currently ~30s)
- **4K @ 4096 iter, 4x4 SS:** <15 seconds (currently ~2 minutes)
- **8K @ 4096 iter, 8x8 SS:** <60 seconds (currently >10 minutes)

### Validation
- Benchmark on mid-range GPU (NVIDIA GTX 1660 or AMD RX 5600)
- Compare with specialized fractal software (XaoS, Kalles Fraktaler)

---

## Documentation Requirements

### User-Facing
- GPU requirements and compatibility list
- Troubleshooting GPU initialization failures
- Performance comparison guide (when to use GPU vs CPU)
- FAQ: "Why is GPU slower for preview?" (initialization overhead)

### Developer-Facing
- Shader architecture documentation
- Adding new fractal shaders guide
- GPU debugging tips (validation layers, shader printf)
- Profiling GPU performance (nsight, renderdoc) 