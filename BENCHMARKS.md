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

## Version 0.1.3 Baseline (December 2024)

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

## Running Benchmarks

To run benchmarks yourself:

```powershell
cargo bench --bench fractal_bench
```

This will run the full benchmark suite (takes ~2-3 minutes) and output results to the console. Copy the relevant sections into this document for the new version.

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
