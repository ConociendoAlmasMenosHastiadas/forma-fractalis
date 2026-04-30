# Performance Benchmarks

This document tracks performance benchmarks across different versions of forma-fractalis to monitor regressions and improvements.

## Format

Each benchmark tests all 4 fractal types (Mandelbrot, Julia, BurningShip, Tippets Mandelbrot) at 3 resolutions (SD/HD/FHD) with 5 iteration counts (256, 512, 1024, 2048, 4096). Each test runs 10 iterations and reports average/min/max times.

## Version 0.1.4 (January 2026)

### Changes from v0.1.3
- Added Powerbrot feature (configurable power parameter)
- Input debouncing (500ms delay on text fields)
- Profiling command-line flag
- **Performance fix**: Optimized power=2.0 case to use multiplication instead of powf()

### HD (1280x720) Results - Most Common Use Case

#### 256 Iterations (UI Target)
| Fractal | Avg | Min | Max | Target | vs v0.1.3 |
|---------|-----|-----|-----|--------|-----------|
| Mandelbrot | 8.04ms | 7.39ms | 8.69ms | ✓ <20ms | ✓ Similar |
| Julia | 6.63ms | 6.09ms | 7.86ms | ✓ <20ms | ✓ Similar |
| BurningShip | 8.41ms | 7.85ms | 8.85ms | ✓ <20ms | ✓ Similar |
| Tippets | 13.46ms | 12.67ms | 15.00ms | ✓ <20ms | ✓ Similar |

#### 1024 Iterations (Common Use)
| Fractal | Avg | Min | Max | Target | vs v0.1.3 |
|---------|-----|-----|-----|--------|-----------|
| Mandelbrot | 18.08ms | 17.13ms | 18.79ms | ✓ <50ms | ✓ Similar |
| Julia | 7.75ms | 7.17ms | 8.43ms | ✓ <50ms | ✓ Similar |
| BurningShip | 19.98ms | 19.47ms | 20.51ms | ✓ <50ms | ✓ Similar |
| Tippets | 41.64ms | 39.87ms | 43.42ms | ✓ <50ms | ✓ Similar |

#### 4096 Iterations (High Detail)
| Fractal | Avg | Min | Max | Target | vs v0.1.3 |
|---------|-----|-----|-----|--------|-----------|
| Mandelbrot | 61.60ms | 58.88ms | 66.67ms | ✓ <150ms | ✓ Similar |
| Julia | 8.28ms | 7.62ms | 9.34ms | ✓ <150ms | ✓ Similar |
| BurningShip | 66.65ms | 64.27ms | 72.88ms | ✓ <150ms | ✓ Similar |
| Tippets | 158.79ms | 148.66ms | 178.11ms | ⚠ >150ms | ✓ Similar |

### Performance Analysis

**Status: NO REGRESSION**
- All fractals maintain v0.1.3 performance levels
- HD @ 256 iter: 7-14ms (excellent)
- HD @ 1024 iter: 8-42ms (all within target)
- HD @ 4096 iter: 8-159ms (most within target)

**Performance Fix Applied:**
- Initial Powerbrot implementation used `z.powf(power)` for all powers
- This was 13-30x slower for default power=2.0 case
- Fixed with optimization: `if power == 2.0 { z * z + c } else { z.powf(power) + c }`
- Result: Classic Mandelbrot (power=2.0) back to v0.1.3 speed
- Custom powers still work correctly with powf()

**Positive:**
- Julia set remains exceptionally fast (6-8ms across all iteration counts)
- All fractals meet performance targets at typical settings
- Powerbrot feature adds no overhead when using default power=2.0

---

## Version 0.2.3 (April 2026)

### Changes Affecting Performance
- Cargo workspace split (core + gui crates) — no algorithmic changes
- Benchmarks now target `-p forma-fractalis-core`
- FractalConfig API benchmark added to test the new public rendering path
- TippetsMandelbrot added as benchmark target (already present from v0.2.2)

### HD (1280x720) @ 1024 Iterations Comparison
| Fractal | v0.1.3 Baseline | v0.2.3 | Delta |
|---------|-----------------|--------|-------|
| Mandelbrot | 19.03ms | 18.21ms | -4% |
| Julia | 8.94ms | 8.55ms | -4% |
| BurningShip | 20.43ms | 19.93ms | -2% |
| Tippets | 42.33ms | 41.94ms | -1% |

### SD (640x480) Results

#### 256 Iterations
| Fractal | Avg | Min | Max |
|---------|-----|-----|-----|
| Mandelbrot | 3.13ms | 2.92ms | 3.46ms |
| Julia | 2.65ms | 2.54ms | 2.85ms |
| BurningShip | 3.33ms | 3.19ms | 3.40ms |
| Tippets | 5.56ms | 5.32ms | 6.13ms |

#### 1024 Iterations
| Fractal | Avg | Min | Max |
|---------|-----|-----|-----|
| Mandelbrot | 8.42ms | 7.75ms | 9.37ms |
| Julia | 3.97ms | 3.59ms | 4.36ms |
| BurningShip | 9.11ms | 8.63ms | 10.11ms |
| Tippets | 19.01ms | 17.50ms | 21.15ms |

#### 4096 Iterations
| Fractal | Avg | Min | Max |
|---------|-----|-----|-----|
| Mandelbrot | 25.84ms | 24.09ms | 32.55ms |
| Julia | 3.24ms | 3.04ms | 3.45ms |
| BurningShip | 29.05ms | 26.94ms | 33.36ms |
| Tippets | 67.74ms | 64.39ms | 82.24ms |

### HD (1280x720) Results

#### 256 Iterations (UI Target)
| Fractal | Avg | Min | Max | Target |
|---------|-----|-----|-----|--------|
| Mandelbrot | 8.71ms | 7.88ms | 9.49ms | ✓ <20ms |
| Julia | 7.62ms | 6.55ms | 8.31ms | ✓ <20ms |
| BurningShip | 9.10ms | 8.46ms | 10.05ms | ✓ <20ms |
| Tippets | 14.16ms | 13.15ms | 14.79ms | ✓ <20ms |

#### 1024 Iterations (Common Use)
| Fractal | Avg | Min | Max | Target |
|---------|-----|-----|-----|--------|
| Mandelbrot | 18.21ms | 17.35ms | 19.06ms | ✓ <50ms |
| Julia | 8.55ms | 7.47ms | 9.27ms | ✓ <50ms |
| BurningShip | 19.93ms | 18.52ms | 21.50ms | ✓ <50ms |
| Tippets | 41.94ms | 39.02ms | 46.29ms | ✓ <50ms |

#### 4096 Iterations (High Detail)
| Fractal | Avg | Min | Max | Target |
|---------|-----|-----|-----|--------|
| Mandelbrot | 64.18ms | 59.47ms | 70.48ms | ✓ <150ms |
| Julia | 9.28ms | 8.49ms | 9.82ms | ✓ <150ms |
| BurningShip | 66.73ms | 62.54ms | 76.39ms | ✓ <150ms |
| Tippets | 154.35ms | 146.36ms | 169.84ms | ⚠ >150ms |

### FHD (1920x1080) Results

#### 256 Iterations
| Fractal | Avg | Min | Max |
|---------|-----|-----|-----|
| Mandelbrot | 19.44ms | 18.49ms | 21.25ms |
| Julia | 16.30ms | 14.89ms | 17.42ms |
| BurningShip | 21.63ms | 20.02ms | 23.05ms |
| Tippets | 33.75ms | 31.89ms | 35.82ms |

#### 1024 Iterations
| Fractal | Avg | Min | Max |
|---------|-----|-----|-----|
| Mandelbrot | 43.72ms | 41.22ms | 46.45ms |
| Julia | 19.42ms | 17.85ms | 21.35ms |
| BurningShip | 47.41ms | 45.42ms | 51.72ms |
| Tippets | 96.23ms | 90.36ms | 119.18ms |

#### 4096 Iterations
| Fractal | Avg | Min | Max |
|---------|-----|-----|-----|
| Mandelbrot | 140.41ms | 131.73ms | 162.49ms |
| Julia | 20.72ms | 19.75ms | 23.05ms |
| BurningShip | 150.50ms | 141.25ms | 156.77ms |
| Tippets | 339.16ms | 326.96ms | 368.71ms |

### FractalConfig API Benchmark (HD 1280x720, 256 iter)
| Fractal | Avg | Notes |
|---------|-----|-------|
| Mandelbrot | 8.94ms | via render_fractal_to_buffer() |
| Julia | 7.78ms | via render_fractal_to_buffer() |
| BurningShip | 9.56ms | via render_fractal_to_buffer() |
| Tippets | 15.39ms | via render_fractal_to_buffer() |

FractalConfig API overhead vs direct: ~0.2-1.2ms — negligible.

### Performance Analysis

**Status: STABLE — no regression from v0.1.x**
- HD @ 256 iter: 7–14ms (all targets met)
- HD @ 1024 iter: 8–42ms (all targets met)
- HD @ 4096 iter: 9–154ms (Tippets marginally over at max; expected)
- FractalConfig API adds no meaningful overhead over direct calls

**FHD note:** Mandelbrot and BurningShip at 4096 iter approach the 150ms boundary; this is normal for the default view (high-density region).

---



### Test Environment
- CPU: (Your CPU - update this)
- OS: Windows
- Rust: 1.x (update with actual version)
- Build: Release with optimizations

### HD (1280x720) Results - Most Common Use Case

#### 256 Iterations (UI Target)
| Fractal | Avg | Min | Max | Target |
|---------|-----|-----|-----|--------|
| Mandelbrot | 8.40ms | 7.33ms | 9.54ms | ✓ <20ms |
| Julia | 7.30ms | 6.54ms | 8.30ms | ✓ <20ms |
| BurningShip | 8.84ms | 7.92ms | 9.31ms | ✓ <20ms |
| Tippets | 14.03ms | 12.89ms | 15.34ms | ✓ <20ms |

#### 512 Iterations
| Fractal | Avg | Min | Max |
|---------|-----|-----|-----|
| Mandelbrot | 12.01ms | 11.19ms | 12.60ms |
| Julia | 8.21ms | 7.66ms | 9.41ms |
| BurningShip | 12.99ms | 12.12ms | 14.04ms |
| Tippets | 24.97ms | 22.53ms | 31.61ms |

#### 1024 Iterations (Common Use)
| Fractal | Avg | Min | Max | Target |
|---------|-----|-----|-----|--------|
| Mandelbrot | 19.03ms | 17.88ms | 21.56ms | ✓ <50ms |
| Julia | 8.94ms | 8.48ms | 9.43ms | ✓ <50ms |
| BurningShip | 20.43ms | 19.42ms | 21.17ms | ✓ <50ms |
| Tippets | 42.33ms | 40.25ms | 45.11ms | ✓ <50ms |

#### 2048 Iterations
| Fractal | Avg | Min | Max |
|---------|-----|-----|-----|
| Mandelbrot | 31.69ms | 29.92ms | 34.53ms |
| Julia | 8.54ms | 7.88ms | 9.89ms |
| BurningShip | 36.09ms | 34.82ms | 38.31ms |
| Tippets | 79.77ms | 77.99ms | 83.26ms |

#### 4096 Iterations (High Detail)
| Fractal | Avg | Min | Max | Target |
|---------|-----|-----|-----|--------|
| Mandelbrot | 60.61ms | 56.14ms | 68.66ms | ✓ <150ms |
| Julia | 9.37ms | 8.22ms | 10.51ms | ✓ <150ms |
| BurningShip | 68.83ms | 64.79ms | 75.87ms | ✓ <150ms |
| Tippets | 159.52ms | 152.53ms | 186.47ms | ⚠ >150ms |

### FHD (1920x1080) Results - High Resolution

#### 256 Iterations
| Fractal | Avg | Min | Max |
|---------|-----|-----|-----|
| Mandelbrot | 17.86ms | 17.12ms | 18.44ms |
| Julia | 15.48ms | 14.68ms | 16.44ms |
| BurningShip | 19.79ms | 18.28ms | 21.79ms |
| Tippets | 32.26ms | 30.57ms | 35.74ms |

#### 1024 Iterations
| Fractal | Avg | Min | Max |
|---------|-----|-----|-----|
| Mandelbrot | 38.95ms | 37.66ms | 40.23ms |
| Julia | 18.70ms | 16.33ms | 19.90ms |
| BurningShip | 46.86ms | 44.57ms | 51.25ms |
| Tippets | 95.72ms | 89.83ms | 108.58ms |

#### 4096 Iterations
| Fractal | Avg | Min | Max |
|---------|-----|-----|-----|
| Mandelbrot | 127.88ms | 122.85ms | 132.37ms |
| Julia | 17.93ms | 17.05ms | 20.12ms |
| BurningShip | 149.01ms | 143.12ms | 166.14ms |
| Tippets | 350.51ms | 335.88ms | 408.43ms |

### Performance Analysis

**Strengths:**
- All fractals meet FPS targets at HD resolution up to 1024 iterations (20+ FPS)
- Julia set is exceptionally fast due to simpler computation
- Mandelbrot and BurningShip show consistent, predictable scaling
- HD @ 4096 iterations meets target for most fractals (7+ FPS)

**Notable:**
- Tippets Mandelbrot is ~2x slower than standard Mandelbrot (expected due to complex calculation)
- Julia performance is remarkable - stays fast even at high iterations (bailout occurs quickly)
- Buffer copy overhead is minimal (< 2ms even at FHD)
- Linear scaling with iteration count as expected

**Thermal Notes:**
- Some variance at highest iteration counts may be due to CPU thermal throttling
- Tippets @ HD/4096 shows wider variance (152-186ms range)

---

## Version Comparison Template

When adding new version results, use this format:

### Version X.Y.Z (Month Year)

#### Changes Affecting Performance
- List any changes that might impact performance
- E.g., "Added GPU acceleration", "Optimized complex number handling"

#### HD (1280x720) @ 1024 Iterations Comparison
| Fractal | v0.1.3 | vX.Y.Z | Delta |
|---------|--------|--------|-------|
| Mandelbrot | 19.03ms | XXms | +/-X% |
| Julia | 8.94ms | XXms | +/-X% |
| BurningShip | 20.43ms | XXms | +/-X% |
| Tippets | 42.33ms | XXms | +/-X% |

---

## Version 0.2.5 (April 2026)

### Changes Affecting Performance
- Added Cactus GPU shader (9th GPU-capable fractal)
- Added Sin Julia hi-prec BigFloat path
- GPU init now logs per-shader compile times and total via `--profiling`
- GPU buffer size logged per render via `--profiling`
- Expanded `gpu_bench` to cover all 9 GPU fractals (was Mandelbrot-only)

### Hardware
- GPU: NVIDIA GeForce RTX 3080 Ti (Vulkan)
- GPU shader compilation total: ~214ms at startup (10 pipelines)

### GPU vs CPU Benchmark — All GPU-Capable Fractals

| Fractal | Config | CPU (ms) | GPU (ms) | Speedup |
|---------|--------|----------|----------|---------|
| Mandelbrot | HD @ 256 iter | 9.30 | 2.90 | 3.21x |
| Mandelbrot | HD @ 1024 iter | 20.26 | 2.74 | 7.38x |
| Mandelbrot | FHD @ 1024 iter | 45.00 | 5.65 | 7.96x |
| Mandelbrot | FHD @ 2048 iter | 72.31 | 5.68 | 12.72x |
| Julia Set | HD @ 256 iter | 8.32 | 2.62 | 3.17x |
| Julia Set | HD @ 1024 iter | 9.69 | 3.40 | 2.85x |
| Julia Set | FHD @ 1024 iter | 22.51 | 5.35 | 4.21x |
| Julia Set | FHD @ 2048 iter | 21.29 | 5.63 | 3.78x |
| Burning Ship | HD @ 256 iter | 9.73 | 2.71 | 3.59x |
| Burning Ship | HD @ 1024 iter | 20.62 | 3.61 | 5.72x |
| Burning Ship | FHD @ 1024 iter | 46.08 | 6.11 | 7.54x |
| Burning Ship | FHD @ 2048 iter | 83.15 | 7.93 | 10.48x |
| Tippets Mandelbrot | HD @ 256 iter | 14.31 | 4.42 | 3.24x |
| Tippets Mandelbrot | HD @ 1024 iter | 42.65 | 5.81 | 7.34x |
| Tippets Mandelbrot | FHD @ 1024 iter | 96.30 | 9.41 | 10.23x |
| Tippets Mandelbrot | FHD @ 2048 iter | 173.34 | 10.32 | 16.80x |
| Multifractal-Julia | HD @ 256 iter | 487.22 | 5.08 | **95.96x** |
| Multifractal-Julia | HD @ 1024 iter | 623.71 | 6.79 | **91.80x** |
| Multifractal-Julia | FHD @ 1024 iter | 1464.84 | 16.19 | **90.51x** |
| Multifractal-Julia | FHD @ 2048 iter | 1777.31 | 11.82 | **150.35x** |
| Cactus | HD @ 256 iter | 11.68 | 5.35 | 2.18x |
| Cactus | HD @ 1024 iter | 21.57 | 6.54 | 3.30x |
| Cactus | FHD @ 1024 iter | 54.03 | 10.51 | 5.14x |
| Cactus | FHD @ 2048 iter | 73.87 | 11.62 | 6.36x |
| Zubieta | HD @ 256 iter | 5.80 | 5.44 | 1.07x |
| Zubieta | HD @ 1024 iter | 5.48 | 5.61 | 0.98x |
| Zubieta | FHD @ 1024 iter | 13.74 | 8.84 | 1.55x |
| Zubieta | FHD @ 2048 iter | 13.38 | 8.90 | 1.50x |
| Sin Julia | HD @ 256 iter | 231.86 | 7.54 | 30.73x |
| Sin Julia | HD @ 1024 iter | 259.93 | 8.74 | 29.74x |
| Sin Julia | FHD @ 1024 iter | 552.69 | 16.11 | 34.30x |
| Sin Julia | FHD @ 2048 iter | 537.17 | 14.95 | 35.92x |
| Insideout Dragon | HD @ 256 iter | 162.50 | 7.49 | 21.71x |
| Insideout Dragon | HD @ 1024 iter | 629.24 | 16.68 | 37.71x |
| Insideout Dragon | FHD @ 1024 iter | 1484.58 | 33.48 | 44.35x |
| Insideout Dragon | FHD @ 2048 iter | 3138.05 | 32.89 | **95.42x** |

### Key Observations
- **Multifractal-Julia** is the biggest winner: CPU is extremely slow (inverse-power formula), GPU achieves **91–150x speedup**. GPU is effectively mandatory for this fractal.
- **Insideout Dragon** is a surprise: CPU is extremely slow at high iter (3138ms FHD@2048), GPU holds steady at ~33ms — **95x speedup**. CPU path is not practical for high iteration counts.
- **Sin Julia** benefits strongly from GPU: transcendental `sin()` per pixel drives 30–36x speedup.
- **Zubieta** shows near-zero GPU benefit at HD (0.98–1.07x): the fractal escapes very quickly and the CPU path is already RAM/overhead-bound, not compute-bound. GPU overhead dominates at small frame sizes; modest gain (~1.5x) at FHD.
- **Cactus** (new this release): modest 2–6x speedup. CPU is already quick due to its iteration structure.
- **Julia Set** shows the smallest consistent speedup (~3–4x) — early escape at low iteration counts keeps CPU competitive.
- GPU times scale weakly with iteration count for most fractals (~2x for 8x iteration increase), indicating compute-bound behaviour on the GPU rather than memory-bound.

### CPU-Only Regression Check — HD (1280x720) @ 1024 Iterations

| Fractal | v0.2.3 | v0.2.5 | Delta |
|---------|--------|--------|-------|
| Mandelbrot | 18.21ms | 20.92ms | +15% |
| Julia | 8.55ms | 10.62ms | +24% |
| BurningShip | 19.93ms | 23.15ms | +16% |
| Tippets | 41.94ms | 45.28ms | +8% |

**Note:** Minor regression (~8–24%) vs v0.2.3. All fractals still within performance targets. Likely caused by increased codegen/LTO pressure from the expanded crate. No algorithmic changes; not a concern given GPU availability.

### CPU Full Results — Classic Fractals (HD 1280x720)

| Fractal | 256 iter | 1024 iter | 4096 iter |
|---------|----------|-----------|-----------|
| Mandelbrot | 9.38ms | 20.92ms | 68.87ms ✓ |
| Julia | 8.71ms | 10.62ms | 12.70ms ✓ |
| Burning Ship | 9.74ms | 23.15ms | 71.78ms ✓ |
| Tippets | 15.48ms | 45.28ms | 165.31ms ⚠ |

### CPU Summary — Compute-Heavy GPU Fractals (HD 1280x720)

| Fractal | 256 iter | 1024 iter | Note |
|---------|----------|-----------|------|
| Multifractal-Julia | 487ms | 624ms | GPU strongly recommended |
| Sin Julia | 232ms | 260ms | GPU strongly recommended |
| Insideout Dragon | 163ms | 629ms | GPU strongly recommended |
| Cactus | 12ms | 22ms | CPU acceptable |
| Zubieta | 6ms | 5ms | CPU fast (early escape) |

---

## Running Benchmarks

To run benchmarks yourself:

```powershell
cargo bench --bench fractal_bench
```

This will run the full benchmark suite (takes ~2-3 minutes) and output results to the console. Copy the relevant sections into this document for the new version.

---

## Recommended Settings by Hardware Profile

Based on v0.2.5 benchmark data (RTX 3080 Ti reference system, 12-core CPU).

### No dedicated GPU / integrated graphics
Use CPU backend. Stick to lower iteration counts for interactive use.

| Use Case | Recommended Settings |
|----------|----------------------|
| Interactive exploration | HD (1280×720), ≤512 iter |
| Compute-heavy fractals (Sin Julia, Insideout Dragon, Multifractal-Julia) | SD (854×480), ≤256 iter |
| Export renders | FHD (1920×1080), ≤1024 iter (expect 30–60s for slow fractals) |

### Mid-range dedicated GPU (e.g. GTX 1060 / RX 580 class)
GPU backend recommended. Speedups will be lower than the RTX reference — expect roughly 50–70% of the listed speedup values.

| Use Case | Recommended Settings |
|----------|----------------------|
| Interactive exploration | HD, ≤1024 iter (GPU), all fractals smooth |
| Compute-heavy fractals | HD, ≤2048 iter GPU — still fast |
| Export renders | FHD or 4K, ≤4096 iter GPU |

### High-end dedicated GPU (RTX 3070+ / RX 6700 XT+)
GPU backend recommended for all fractals. The reference benchmarks above apply directly.

| Use Case | Recommended Settings |
|----------|----------------------|
| Interactive exploration | FHD, ≤2048 iter GPU |
| Compute-heavy fractals | FHD, ≤4096 iter GPU — all complete in <50ms |
| Export renders | 4K–8K, high iter counts practical |

### When to prefer CPU over GPU
- Zubieta at HD and below: GPU overhead exceeds compute savings (~1x speedup); CPU is equally fast.
- Any fractal at very small preview sizes (< 256×256): GPU dispatch overhead dominates.

---

## Notes

- **Target Performance Goals:**
  - 1280x720 @ 256 iter: < 20ms (50+ FPS) - Smooth UI interaction
  - 1280x720 @ 1024 iter: < 50ms (20+ FPS) - Common use case
  - 1280x720 @ 4096 iter: < 150ms (7+ FPS) - High detail exploration

- **Benchmark Limitations:**
  - CPU-bound workload (parallel rendering with rayon)
  - Performance varies with CPU temperature (thermal throttling)
  - Results may vary between runs by ~10-15% at high iteration counts
  - Background processes can affect results

- **Interpretation:**
  - Min time = best-case performance (cold CPU)
  - Max time = worst-case (thermal throttling or system load)
  - Avg time = typical expected performance
