# Changelog

All notable changes to Forma Fractalis will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5] - 2026-04-29

### Added
- **ChaosSymmetry1 Attractor Fractal**: orbit accumulation via z_{n+1} = (a0 + a1|z|² + a2·Re(z^m) + a3·i)·z + a4·conj(z)^{m-1}
  - CPU f64 path: `accumulate_orbits()` with two-pass burn-in and golden-angle sub-orbit seeds
  - CPU hi-prec path: f64 orbit arithmetic + BigFloat screen coordinate mapping
  - GPU f32 orbit compute shader (`chaos_symmetry1_orbit_kernel.wgsl`); repurposes IFS param slots
  - GUI: full-width sliders for a0–a4, TextEdit alongside each for precise entry, m / samples / burn-in / log-density / seed controls
- **Cactus GPU Shader** (`cactus_kernel.wgsl`): escape-time GPU rendering, registered in backend selection; 2–7x speedup over CPU
- **Sin Julia Hi-Precision CPU Support**: BigFloat `sin()` iterate with 5 unit tests
- **`FractalTypeOps::uses_orbit_accumulation()`**: hides Iterations control in GUI for orbit-accumulation fractals (ChaosSymmetry1, Multi-Julia IFS, AdjProbJulia)
- **Hardware Profile Benchmark Guide**: "Recommended Settings by Hardware Profile" section in BENCHMARKS.md

### Changed
- `image` crate upgraded to v0.25; `png` crate upgraded to v0.18
  - `png::Decoder::new` now requires `BufRead + Seek` — wrapped `File` in `BufReader`
  - `Compression::Default` removed — replaced with `Compression::default()`
- Fractal type dropdown in GUI changed to full-width ComboBox (no more truncation)
- GPU test list (`gpu_test.rs`) updated to include Cactus

### Fixed
- Windows SmartScreen "protected your PC" — workaround documented in README under Installation

### Technical
- capability_table.md: ChaosSymmetry1 (all three backends), Cactus GPU, Sin Julia hi-prec all marked yes



### Added
- **Multi-Julia IFS Fractal**: inverse-iteration chaos game via orbit accumulation
  - Algorithm: z = +/-sqrt(z - c_i), map chosen randomly by probability weight each step
  - 2-8 configurable IFS maps with per-map Re/Im and Mag/Angle coordinate modes
  - Linked probability sliders (sum stays 1.0 automatically)
  - Add/remove maps with buttons; seed parameter for reproducible renders
  - `samples`, `burn_in`, and `use_log_density` parameters
- **Orbit Accumulation Infrastructure** (`core/src/orbit_accumulation.rs`):
  - `DensityBuffer` and `AtomicDensityBuffer` for parallel histogram accumulation
  - `OrbitTarget` trait abstracting over both buffer types
  - Deterministic K=64 sub-orbit decomposition seeded via golden-ratio mixing
  - Bit-identical output regardless of thread count
  - Automatic switch to atomic shared buffer above 500 MB threshold (OOM prevention)
- **GPU Orbit Accumulation**: `orbit_common.wgsl` shader template with 1D dispatch
  - Each thread runs one sub-orbit; writes to shared `atomic<u32>` density histogram
  - `multi_julia_ifs_orbit_kernel.wgsl` for Multi-Julia IFS chaos game on GPU
  - CPU-side normalization and coloring after GPU density readback
- **CPU Hi-Precision Orbit Path**: BigFloat complex sqrt + BigFloat coordinate mapping
  - `accumulate_orbits_hiprec()` in Multi-Julia IFS uses astro-float at 64-1024 bits
- **Burning Ship Hi-Precision**: full BigFloat support for all bit widths (64-1024)
- **GPU Status Fix**: GPU indicator now correctly shows "ready" for orbit-accumulation fractals
- **GitHub Sponsors**: FUNDING.yml, support section in README.md and index.html
- **SEO**: repository topics, updated descriptions, og:image and twitter:image updated

### Fixed
- **GPU shader validation panic**: `OrbitParams` uniform buffer used `array<f32, N>` with 4-byte
  stride, violating WGSL uniform alignment rules; switched to `var<storage, read>` binding
- **GPU "not available" false warning**: `supports_fractal()` only checked escape-time pipelines;
  now also checks orbit pipelines via `orbit_pipeline_name_for()`

### Technical
- `Fractal` trait: `uses_orbit_accumulation()`, `accumulate_orbits()`, `accumulate_orbits_hiprec()` methods
- `FractalRenderer` trait: `render_orbit_density()` and `supports_orbit_density()` methods
- Rendering pipeline routes orbit-accumulation fractals before backend dispatch
- capability_table.md: Multi-Julia IFS CPU=yes, hi-prec=yes, GPU=yes; Burning Ship hi-prec=yes

## [0.2.3] - 2026-04-03

### Added
- **Cargo Workspace Architecture**: Project split into two crates
  - `forma-fractalis-core` — pure Rust library with no GUI dependencies (fractals, rendering, GPU, hi-prec, export)
  - `forma-fractalis` — GUI application that depends on core
  - Core library intended for headless rendering, CLI tools, and future crates.io publication
- **Public Core Library API**: High-level, ergonomic API for library consumers
  - `render_fractal_to_buffer(fractal, config)` — single-call rendering
  - `compute_fractal_iterations(fractal, config)` — phase 1: iteration counts only
  - `colorize_iterations(iterations, color_config)` — phase 2: apply colormap without re-rendering
  - `FractalConfig`, `ColorConfig`, `ExportConfig` builder types in `core/src/config.rs`
  - `FractalView` builder methods: `with_center()`, `with_zoom()`, `with_view_parameter()`
  - `FractalIterations` struct: pre-computed iteration cache with `is_valid_for()` cache check
- **Two-Phase Rendering Pipeline**: Compute iterations once, recolor cheaply
  - GUI uses `FractalIterations` cache — color-only changes (colormap drag, period, log scale) skip re-render
  - Replaces hand-rolled `IterationCache` struct with library type
  - Enables color animation loops without re-running fractal math
- **Color Offset**: `color_offset: u32` in `ColorConfig`
  - Shifts colormap lookup by a fixed number of period steps
  - GUI slider visible when period modulation is enabled
  - `with_color_offset()` builder for animation loops
- **PowerJulia Fractal** (release fractal): exponent parameter `k` added to Julia Set
  - Iteration: z_{n+1} = z_n^k + c
  - GUI slider (range 1–8) and advanced text input for k
  - Default k=2 preserves classic Julia behaviour; existing saves load with k=2
- **Julia Set: CPU Hi-Precision**: Full BigFloat support for all power values
  - Power=2 path uses direct BigFloat multiplication (no transcendentals)
  - General power uses polar form: `r^k * (cos(k*theta) + i*sin(k*theta))`
  - Private `bf_atan2()` and `complex_powf_bf()` helpers
  - 5 hi-prec tests added (unit-disk interior, escape, f64 agreement, power=3, bit-width smoke)
- **Julia Set: GPU power parameter**: `param_2 = power` in `julia_kernel.wgsl`
  - Fast path for power=2 (direct `complex_mul`), general path via `complex_pow`
- **TippetsMandelbrot GPU Shader**: GPU acceleration for Tippets Mandelbrot
  - Implements the sequential scalar update algorithm: x_new = x²-y²+a, y = 2*x_new*y+b
  - Fixed a pre-existing incorrect shader formula (was using z²+cz+c, wrong algorithm)
- **GPU Test Automation**: `cargo run --release -- gpu-test` CLI subcommand
  - Renders each GPU-supported fractal with both GPU and CPU backends
  - Compares raw iteration counts (±1 tolerance for f32/f64 boundary pixels, <5% mismatch allowed)
  - Saves PNG images to `temp/gpu_test/` for visual inspection on failure, cleaned on success
  - 7 fractals validated: Mandelbrot, Julia Set, Burning Ship, Insideout Dragon, Zubieta, Sin Julia, Tippets Mandelbrot
  - `--fractal NAME` flag for testing a single fractal
- **Core Library Examples**: `core/examples/simple_render.rs`, `core/examples/batch_export.rs`
- **Core Library Documentation**: `core/README.md` with quick start, API overview, fractal table

### Changed
- **GUI Cache**: `IterationCache` replaced by `FractalIterations` from core library
- **Tippets Mandelbrot equation string**: corrected to accurately describe the sequential update algorithm
- **GPU test comparison**: switched from mean-luminance RGBA comparison to raw iteration count comparison — eliminates colormap-amplification false negatives on dark fractals (Zubieta)

### Fixed
- **Tippets Mandelbrot GPU shader**: was computing z²+cz+c instead of the correct sequential scalar algorithm; GPU and CPU renders now match
- **Julia GUI**: removed spurious separator before the Exponent section

### Technical
- **AGENTS.md**: mandatory three-backends rule for new fractals (CPU f64 + CPU Hi-Prec + GPU); applies to parameter additions on existing fractals
- **AGENTS.md**: index.html 4-step release checklist for showcase image updates
- **Capability table**: Julia hi-prec=yes; TippetsMandelbrot GPU=yes; hi-prec rollout schedule v0.2.4–v0.3.2
- **Testing**: 115 tests passing (106 core + 9 GUI, up from 95+9)
  - 5 Julia hi-prec tests (unit-disk, escape, f64 agreement, power=3, bit-width smoke)
  - GPU test suite covers 7 fractals automatically

### Migration (v0.2.2 → v0.2.3)
The GUI application (`forma-fractalis`) has the same public interface and all imports remain compatible via re-exports in `gui/src/lib.rs`. No changes required for GUI users.

Library users (direct rendering code): use `forma-fractalis-core` for headless rendering:
```rust
use forma_fractalis_core::{fractals::Mandelbrot, config::FractalConfig,
    fractals::FractalView, render_fractal_to_buffer};
let fractal = Mandelbrot::new();
let config = FractalConfig::new(FractalView::new(1920, 1080), 256);
let buffer = render_fractal_to_buffer(&fractal, &config)?;
```

## [0.2.2] - 2026-03-28

### Added
- **Insideout Dragon Fractal**: z_{n+1} = z_n^2 + f(|z_n|) + i*g(|z_n|), z_0 = 1/c
  - Magnitude-based perturbation via f(r) and g(r) rational functions
  - Configurable escape radius with GUI controls
  - Numerical stability guards (singularity, NaN/Inf, near-zero denominator)
  - CPU and GPU rendering support
  - Hi-precision support (BigFloat z^2 accumulation, f64 perturbation terms)
- **CPU High-Precision Pipeline**: Arbitrary-precision rendering via astro-float BigFloat
  - User-selectable bit width: 64, 128, 256, 512, 1024 bits
  - CpuHiPrec backend alongside CPU and GPU in Performance section
  - BigFloat coordinate pipeline for clean rendering at extreme zoom (>1e15)
  - Mandelbrot: full power support via polar form complex exponentiation
  - Insideout Dragon: hi-prec with f64 perturbation fallback
  - Warning displayed for fractals without hi-prec support
- **HSV Color Picker**: Replaces RGB slider triplets with interactive visual picker
  - 2D saturation-value plane with click/drag interaction
  - Horizontal hue bar with full spectrum selection
  - Hex input field (6-character, with or without #)
  - Editable R/G/B text inputs for direct numeric entry
  - Live color swatch preview
  - Used for both interior color and colormap stop editing
- **Burning Ship GPU Shader**: GPU acceleration for Burning Ship fractal
  - burning_ship_kernel.wgsl with abs() on both components
- **CPU Thread Limit**: User-controlled thread count for CPU/CpuHiPrec backends
  - Slider in Performance section (1 to max cores, 0 = all cores)
  - Local rayon ThreadPool with graceful fallback
- **Preview Zoom Control**: Manual preview scale (0.1x-1.0x)
  - Auto-halved to 0.5x when entering CpuHiPrec mode
  - Restored on exit from CpuHiPrec

### Changed
- **Color Editing UI**: RGB sliders removed in favor of HSV color picker
  - Interior color section uses chromator-inspired HSV picker
  - Colormap stop editor uses same picker widget
  - Both hex and RGB text entry available for precision

### Fixed
- **Export Pipeline**: hiprec_bits and max_threads now propagated to PNG export
  - RenderConfig builder chains .with_hiprec_bits() and .with_max_threads()
  - Both GUI export paths (GPU/non-GPU) pass new parameters

### Technical
- **Dependencies Updated**:
  - astro-float: v0.9 added (arbitrary-precision floating point)
- **Testing**: 95 tests passing (up from 82)
  - 5 Mandelbrot hi-prec tests (origin, escape, f64 agreement, bit widths, general power)
  - 4 Insideout Dragon hi-prec tests (origin, escape, f64 agreement, bit widths)
  - 4 color picker tests (HSV roundtrip, hex parsing, state management)

## [0.2.1] - 2026-03-09

### Added
- **Sin Julia Fractal**: z_{n+1} = c·sin(z_n)
  - Hyperbolic decomposition: x_{n+1} = sin(x)cosh(y), y_{n+1} = cos(x)sinh(y)
  - Parameters: c (real/imaginary) with rectangular and polar input modes
  - Configurable escape radius (4.0 to 200.0, default 50.0)
  - Full GPU acceleration via sin_julia_kernel.wgsl
  - sinh/cosh utilities added to common.wgsl
  - GPU infrastructure extended: param_2 added to GpuFractalParams
- **Animated GIF Export**: Generate zoom sequences and iteration-fade animations
  - Zoom animation: logarithmic interpolation between two zoom levels
  - Iteration fade: animate from low to high iteration counts
  - Configurable frame count and FPS
  - GPU-accelerated frame rendering (uses selected backend, initialized once per run)
  - Cancel button: stops generation mid-run, deletes partial file
  - Export settings (scale, filter, directory) inherited from PNG export section
  - Per-frame profiling with zoom/center or iteration details
  - GIF filename convention matches PNG: animation_{fractal}_{W}x{H}{filter}_{timestamp}.gif
- **Load from JSON**: Settings files loadable alongside PNG files
  - "Load from PNG or JSON" dialog accepts .png and .json
  - Routes by file extension; unsupported extensions produce a clear error
- **Julia GPU Acceleration**: Julia Set now renders via compute shader
  - julia_kernel.wgsl with c as uniform parameter (param_0/param_1)
- **Logarithmic Zoom Interpolation**: Perceptually uniform zoom sequences
  - Each frame multiplies zoom by the same ratio (geometric mean interpolation)
  - Replaces linear lerp that produced visually uneven pacing
- **Colorstop Period Fix**: scala-chromatica updated to v0.1.4
  - Inclusive sampling formula: (iter % period) / (period - 1)
  - Endpoint colors (position=1.0 stops) now correctly sampled
  - Fixes Egyptian Echo and similar colormaps not showing endpoint colors

### Changed
- **Animation UI**: Julia Parameter Sweep hidden from dropdown (deferred to v0.3.x)
  - Pending project variables system; returns explicit error if triggered via old state
- **Import Section**: Renamed from "Import from PNG" to "Import"
  - File dialog accepts both .png and .json with combined filter
- **GPU/CPU Selector**: Changed from ComboBox to radio buttons
  - Fewer clicks, clearer selection state
- **Animation Output Directory**: Merged with PNG export directory
  - Single "Choose Directory" button in Export section
  - Animation reuses export directory with warning if unset
- **GIF Filename Convention**: Aligned with PNG convention
  - Removed chrono dependency (replaced with std::time::SystemTime)

### Fixed
- **Animation Color Settings**: Color modulation never reached animation frames
  - use_period, period, use_interior_color, interior_color, use_log_scale now threaded through full call chain
- **Animation Export Scale**: export_scale was silently ignored in animation frames
- **Animation GIF Dimensions**: Width/height assertion panic on dimension mismatch fixed
  - AnimationConfig now uses scaled output dimensions to match render_frame

### Technical
- **Dependencies Updated**:
  - scala-chromatica: v0.1.2 → v0.1.4 (colorstop period fix)
  - chrono: removed (std::time::SystemTime used instead)
- **GPU Infrastructure**: param_2 added to GpuFractalParams for fractals needing 3 parameters
- **Testing**: 82 tests passing (up from 73)
  - Animation color regression test (test_color_settings_affect_frame_output)
  - examples/test_v0_1_4.rs for colorstop period verification

## [0.2.0] - 2026-02-27

### Added
- **GPU Acceleration**: Full compute shader pipeline via WGPU
  - Backend selection: CPU or GPU mode (explicit user control)
  - Mandelbrot and Powerbrot GPU support
  - Compute shaders for parallel iteration calculation
  - Tiled rendering for large exports (handles >256 MB buffers)
  - Automatic buffer limit detection and tile size calculation
  - GPU safety: 500k iteration cap, explicit error handling
  - No silent CPU fallback - errors shown in status bar
  - Performance profiling integration with --profiling flag
  - f32 precision on GPU vs f64 on CPU (visible at deep zoom)
- **Zubieta Fractal**: Julia variant with division operator
  - Formula: z_{n+1} = z_n^2 + c/z_n
  - Division-by-zero guards (epsilon checks)
  - Full CPU and GPU implementation
  - 8 unit tests for iteration and edge cases
- **Polar/Rectangular Coordinate Mode**: Complex parameter input
  - Toggle for Julia and Zubieta c parameter
  - Rectangular mode: Re{c} / Im{c} sliders and text inputs
  - Polar mode: |c| (0-3) / ang(c) (0-2π) sliders and text inputs
  - Real-time coordinate conversion
  - 15-digit precision text inputs
  - Normalized atan2 handling for correct slider behavior
- **Shader Composition Architecture**: Modular GPU shader system
  - common.wgsl: Shared framework (bindings, utilities, entry point)
  - Per-fractal kernel files: Isolated iteration logic
  - Template marker replacement at shader load time
  - Single unified GpuFractalParams struct (param_0/param_1 slots)
  - Eliminates code duplication across fractals
- **Unified Rendering Pipeline**: Single path for preview and export
  - RenderBackend enum with explicit mode selection
  - RenderTarget enum (Preview vs Export with dimensions)
  - Result-based error handling throughout pipeline
  - Identical color application on CPU and GPU
  - Deprecates old iteration cache in ViewState
- **Release Showcase Pattern**: Version-specific fractal images
  - img_resources/showcases/ folder for release images
  - Showcase displayed below release notes in README
  - Previous releases moved to gallery_expanded/
  - Passive gallery growth pattern

### Changed
- **GPU State Management**: Explicit mode selection
  - Removed "Auto" mode (was ambiguous and unpredictable)
  - User chooses CPU or GPU explicitly in Performance section
  - GPU initializes immediately when switched to GPU mode
  - Clear indication which backend is active
- **Error Handling**: No silent failures
  - render_with_config() returns Result<Vec<u8>, String>
  - GPU errors displayed in status bar with fallback prompt
  - User decides whether to switch to CPU after GPU error
  - No automatic silent fallback masking issues
- **Shader Code**: Unified parameters and reduced duplicatoin
  - wgpu_backend.rs: 760 lines -> 482 lines (37% reduction)
  - Single render_fractal() and render_fractal_tiled() for all fractals
  - Eliminated per-fractal parameter structs and render methods
  - Composition-based shader loading and compilation
- **Insideout Dragon**: Deferred to v0.2.2
  - Implementation exists but hidden from GUI
  - FractalType::all() excludes InsideoutDragon
  - Numerical stability issues require deeper investigation
  - Will be added with proper singularity handling in v0.2.2

### Technical
- **Dependencies Added** (optional, default enabled):
  - wgpu 0.19: WebGPU implementation for Rust
  - pollster 0.3: Blocking on async GPU operations
  - bytemuck 1.14: Safe buffer casting for GPU data
- **Dependencies Updated**:
  - scala-chromatica: Git dependency -> 0.1.2 (now published on crates.io)
- **Feature Flag**: `gpu` feature enabled by default
  - Can disable with --no-default-features for CPU-only builds
  - Conditional compilation throughout codebase
- **Testing**: 73 tests passing (up from 71)
  - Zubieta iteration tests
  - GPU safety and buffer limit tests
  - Coordinate conversion tests
  - Integration tests for both CPU and GPU paths
- **Code Architecture Improvements**:
  - Grouped state pattern leveraged for GPU state
  - FractalGUI trait extended for polar/rectangular toggles
  - CoordinateMode enum in app_state.rs
  - InputState extended with magnitude/angle fields
- **WGSL Shader Utilities**:
  - pixel_to_complex() coordinate mapping
  - complex_mul() and complex_pow() helpers
  - Coordinate parity verified between CPU and GPU

### Performance
- **GPU Speedup**: Significant for high-resolution exports
  - Parallel processing thousands of pixels simultaneously
  - Ideal for 4K+ exports with high iteration counts
  - Tiled rendering prevents memory exhaustion
- **Preview Latency**: GPU initialization ~100-200ms
  - Users should manually select GPU mode when desired
  - CPU mode maintains instant preview updates

### Documentation
- **AGENTS.md Updates**:
  - GPU Shader Composition pattern documented
  - Polar Coordinate GUI Pattern with atan2 normalization
  - Release showcase image workflow
  - Added step 6b to "Adding New Fractals" checklist
- **README.md Enhancements**:
  - v0.2.0 release notes and showcase image
  - Zubieta fractal in release showcase
  - Vertical cactus added to expanded gallery

## [0.1.9] - 2026-02-16

### Added
- **Lemon Fractal**: Convergence-based Newton-type fractal
  - Formula: z_{n+1} = z_0 * z_n^2 * (z_n^2 + 1) / (z_n^2 - 1)^k
  - Configurable denominator power (k): slider range -5.0 to 5.0
  - k=2 is canonical Lemon, k=1 is an interesting "typo variant"
  - Convergence threshold parameter (10^-x notation)
  - 8 unit tests for iteration and parameter behavior

### Changed
- **Dependencies Updated**:
  - rayon: 1.8 -> 1.11
  - once_cell: 1.19 -> 1.21
  - clap: 4.4 -> 4.5
  - tempfile: 3.8 -> 3.25
  - rfd: 0.12 -> 0.17
  - indicatif: 0.17 -> 0.18
- **Repository Cleanup**:
  - Removed AGENTS.md from version control (local-only LLM workspace file)
  - Restructured img_resources into banners/, gallery/, gallery_expanded/
  - Improved README banner placement for better visual flow
  - Removed unused `directories` dependency
  - Deleted stray test_load.rs file
- **Documentation**:
  - Removed emojis from CHANGELOG.md and BENCHMARKS.md
  - Trimmed COLORMAP_SAVELOAD.md verbosity
  - Added FractalGUI trait and Dependency Management sections to AGENTS.md
  - Updated future plans with GPU phasing notes

### Fixed
- Fixed test_tetration_default_view assertion (zoom 0.5 -> 0.45)

## [0.1.8] - 2026-02-15

### Added
- **Tetration Fractal**: z_{n+1} = c^(z_n) with expandable escape criteria system
  - Four escape criteria modes: magnitude, real, imaginary, either component
  - Radio button UI for escape mode selection
  - Logarithmic threshold slider (10¹ to 10¹⁰)
  - Scientific notation support for threshold input
  - Default view optimized for interesting structures (zoom=0.45)
- **Fractal Equation Visualization**: Mathematical formulas displayed in GUI
  - LaTeX-style ASCII notation for all 8 fractals
  - Appears below fractal type selector
  - 14pt italic styling for readability
  - Unicode support for Greek letters (φ) where available
- **Iteration Cache System**: Mandatory transparent performance optimization
  - Automatic caching of iteration data after every render
  - Instant color changes (10-20ms vs seconds)
  - Cache invalidation on view/zoom/parameter changes
  - ~4.5 MB memory overhead for preview (1380×820)
  - Completely transparent to users
- **Performance Profiling Flag**: `--profiling` / `-p` flag for debugging
  - Displays `[CACHE]` and `[PERF]` log messages
  - Integrated with clap CLI parser
  - Works in both GUI and CLI render modes
  - Usage: `forma-fractalis --profiling`
- **FractalGUI Trait**: Architectural improvement for parameter rendering
  - Encapsulated parameter UI logic per fractal
  - Removed 240+ lines of hard-coded GUI logic
  - Extensible for future fractal-specific parameters
  - Clean separation of concerns
- **README Enhancements**: High-quality showcase images
  - Hero banner image (4320×1080 Julia Set)
  - Wide fractal images as section headers
  - Gallery with Mandelbrot, Julia, Multifractal-Julia, Tetration
  - All images at 4K resolution with 4× Lanczos scaling

### Changed
- **GUI Resize**: Expanded window for better parameter visibility
  - Window: 1580×750 → 1730×820 (+150px width, +70px height)
  - Sidebar: 300px → 350px (+50px)
  - Display area: ~1280×720 → ~1380×820
  - Maintains ~16:9 aspect ratio for display
- **Iteration Cache**: Made mandatory (was optional)
  - Removed cache enable/disable toggle
  - Removed Performance section GUI controls
  - Simplified rendering logic (~47 lines removed)
  - Single code path: check validity → use or compute+cache
- **Equation Rendering**: Switched from Unicode to LaTeX-style notation
  - Initial Unicode subscripts/superscripts caused rendering issues
  - Now uses z_{n+1} style notation for reliability
  - Cross-platform font compatibility

### Removed
- **Cache Toggle Controls**: Performance section removed from GUI
  - No more "Enable Iteration Cache" checkbox
  - No cache status display
  - No "Clear Cache" button
  - Cache is now transparent and automatic
- **Legacy Rendering Path**: Non-cached rendering removed
  - Unused RenderConfig/RenderTarget imports removed
  - Simplified codebase with single rendering path

### Technical
- **Code Quality**: ~287 lines removed total
  - FractalGUI refactoring: 240 lines
  - Cache simplification: 47 lines
- **Parameter System**: Extensible enum-based parameters
  - Created `parameter_types.rs` for EscapeMode enum
  - Radio button generation from parameter definitions
  - Reusable for future fractals with custom escape criteria
- **Testing**: All features tested and validated
  - Tetration escape modes verified
  - Cache invalidation confirmed
  - Equation rendering across all fractals
  - Performance profiling output validated

### Performance
- **Instant Recoloring**: Color/period/log scale changes are near-instant
- **Parallel Rendering**: Both iteration and coloring parallelized via Rayon
- **Memory Efficiency**: Cache size scales with preview dimensions only

## [0.1.7] - 2026-02-14

### Added
- **Command-Line Rendering**: Headless fractal rendering without GUI
  - `forma-fractalis render --input <file> --output <file>` command
  - Supports loading from PNG metadata or JSON settings files
  - Parameter overrides via CLI (--width, --height, --iterations, --scale, --supersample)
  - Progress indicators with elapsed time display
  - Proper exit codes (0=success, 1=error)
  - Built with `clap` 4.4 for robust argument parsing
- **Export Settings as JSON**: Standalone settings export without rendering
  - "Export Settings (JSON)" button in GUI
  - Creates shareable, human-readable settings files
  - Smaller files for version control and collaboration
  - Can be used with CLI for batch rendering
- **State Conversion System**: Clean bidirectional metadata↔state conversion
  - `From<&FractalMetadata>` implementations for all state structs
  - `FractalMetadata::from_app_state()` for reverse conversion
  - Eliminates 50+ lines of manual field mapping
  - Single source of truth for conversion logic
- **Enhanced Export API**: State-based export function
  - `export_png_from_state()` simplifies export calls
  - Reduced from 13 parameters to 5 state struct references
  - Automatically handles metadata creation
  - Cleaner, more maintainable code architecture

### Changed
- **Refactored State Management**: Improved maintainability
  - `load_from_metadata()` simplified from 50+ lines to 10 lines
  - Uses trait-based conversions instead of manual field mapping
  - Easier to add new fields (one place instead of three)
  - Better type safety and compiler-enforced completeness
- **Main Function**: CLI integration
  - Checks for CLI arguments before launching GUI
  - Maintains backward compatibility with GUI mode
  - Seamless CLI/GUI mode switching

### Removed
- **Completed scala-chromatica Migration**: Removed deprecated bridge modules
  - Deleted `src/colorschemes.rs` bridge module
  - Deleted `src/colorschemes_io.rs` bridge module
  - Deleted `src/colormaps/` directory (14 JSON files)
  - All imports now use `scala_chromatica` directly
  - Deprecation cycle from v0.1.61 complete

### Dependencies
- Added `clap` 4.4 with derive feature for CLI argument parsing
- Added `indicatif` 0.17 for CLI progress indicators

### Testing
- Added 5 new unit tests for state conversions
- Test round-trip metadata → state → metadata
- Verify all fractal types convert correctly
- All 37 tests passing

## [0.1.6] - 2026-01-26

### Added
- **Load from PNG Feature**: Complete round-trip import/export functionality
  - "Load from PNG" button in GUI with file picker dialog
  - Reads all fractal settings from PNG tEXt chunks
  - Restores fractal type, parameters, view, colormap, and all settings
  - Enables sharing discoveries with embedded settings
  - Foundation for command-line rendering (v0.1.7)
- **Cactus Fractal**: New cubic iteration fractal type
  - Formula: z_{n+1} = z_n^3 + (z_0 - 1)z_n - z_0
  - Adaptive escape radius: R > max(1, sqrt(|z_0-1|+1), |z_0|^(1/3))
  - Default view centered at origin with 0.6 zoom
  - 7 comprehensive unit tests covering edge cases
  - Reference: https://paulbourke.net/fractals/cactus/
- **Vintage Lavender Colormap**: 5-color muted earth tones palette
  - Vintage Lavender (#885a89) → Muted Teal (#8aa8a1) → Pale Slate (#cbcbd4)
  - → Tan (#d1b490) → Pumpkin Spice (#ee7b30)
  - Complements Cactus fractal's organic structure
  - Created using coolors.co palette parser
- **Enhanced Metadata Format**:
  - Added `Created-Timestamp` field for organization
  - All metadata fields now properly documented
  - Version fields for future compatibility tracking
- **METADATA_FORMAT.md**: Comprehensive documentation of PNG metadata structure
  - Complete field reference with examples
  - Fractal-specific parameter documentation
  - Future command-line usage examples
  - Technical notes on PNG chunk implementation
- **PNG Metadata Parsing**: `load_png_metadata()` function in export module
  - Validates Forma Fractalis metadata presence
  - Graceful error handling for missing/invalid fields
  - Parses all metadata into `FractalMetadata` struct
  - Helper methods for type conversion (fractal type, filter type, view)
- **State Loading System**: `load_from_metadata()` method in FractalApp
  - Applies all settings from metadata to application state
  - Updates fractal type, parameters, view, colormap, and export settings
  - Synchronizes text input fields with loaded values
  - Triggers automatic redraw after loading
- **Test Coverage**: Comprehensive unit and integration tests
  - 35 tests total (28 existing + 7 new)
  - Integration test: Export → Load → Verify round-trip (test_export_and_load_roundtrip)
  - Integration test: Load into fresh application state (test_load_into_fresh_state)
  - Integration test: State replacement between different fractals (test_state_replacement_different_fractal)
  - Integration test: All 6 fractal types export/load correctly (test_export_and_load_all_fractal_types)
  - Edge case tests: Invalid PNGs, missing metadata, nonexistent files
  - Unit tests: Metadata parsing, type conversions, struct construction
- **PNG Metadata Improvements**: Added View-Width and View-Height fields
  - Stores original view dimensions separately from scaled export dimensions
  - Backward compatible: Falls back to PNG dimensions for old metadata
  - Fixes bug where loaded dimensions were incorrect after scaling

### Fixed
- **Bug Fix**: Width/height now correctly restore from metadata
  - Previously: Loaded PNG image dimensions (scaled) instead of original view size
  - Now: Stores View-Width and View-Height in metadata explicitly
  - Impact: Load from PNG now correctly reproduces original view dimensions
  - Backward compatible with existing PNGs
- **Bug Fix**: Removed outdated "mandelrust" references
  - Updated all documentation to use correct library name "forma_fractalis"
  - Fixed build script output messages
  - Updated COLORMAP_SAVELOAD.md examples
  - Deleted obsolete mandelrust_v*.zip files from builds directory

### Changed
- **Export Module**: Now handles both export and import operations
  - Renamed module documentation to "Image Export and Import System"
  - Added Serde derive traits for serialization
  - Import capabilities complement existing export features

### Documentation
- New METADATA_FORMAT.md with complete format specification
- Updated README.md with Load from PNG feature and Cactus fractal
- Updated version badge to 0.1.6
- Plan v0.1.6 tracking updated with progress notes

### Technical Notes
- Metadata version: 1.0 (first versioned format)
- All state managed through v0.1.5's grouped state pattern
- Error messages use visual feedback indicators
- Status message prefixing ("LOAD_PNG:") for GUI-to-logic communication
- Cactus fractal uses adaptive escape radius for accuracy

### Testing
- All 28 unit tests passing (7 new for Cactus fractal)
- Code compiles without errors or warnings
- Release build successful
- Manual testing confirmed: export -> load -> exact reproduction
- Cactus fractal renders correctly with Vintage Lavender colormap

## [0.1.5] - 2026-01-19

### Added
- **Multifractal-Julia Fractal**: New fractal type with unique cycle detection
  - Formula: z_{n+1} = c^k · z_n^{-2} + c
  - Power parameter k: range -5.0 to 5.0 (default 1.0)
  - Special case optimizations for k = -1, 0, 1, 2
  - General complex power support for non-integer k
  - HashMap-based cycle detection tracks periodic orbits
  - Returns period length when cycles detected
  - Adaptive bailout radius based on power magnitude
- **Cosmic Dawn Colormap**: 5-color gradient palette
  - Deep space blues (Prussian Blue, Space Indigo)
  - Through cosmic purples (Amethyst, Lilac)
  - To dawn pinks (Cotton Rose)
  - Perfect complement to Multifractal-Julia's period-based coloring
- **number_utils Module**: Centralized numerical constants
  - ABSOLUTE_EPSILON = 1e-12 for consistent precision checks
  - Replaced hardcoded epsilon values across fractals

### Changed
- **Rendering Parallelization**: Massive performance improvement
  - Changed from row-level (600 tasks) to pixel-level (480,000 tasks) parallelization
  - Rayon work-stealing now distributes load across all CPU cores efficiently
  - Expect 75-100% CPU utilization on all cores during rendering
  - 2-4x faster rendering on multi-core systems
- **Mandelbrot fractal**: Now uses centralized ABSOLUTE_EPSILON constant

### Fixed
- Low CPU utilization during rendering (was 25%, now 75-100%)
- Inconsistent epsilon values across different fractals

### Documentation
- Updated AGENTS.md with fractal/colormap requirements for each release
- Added "Fractal & Colormap: TBD" notes to all future plans (v0.1.6-v0.2.2)
- Comprehensive test suite for Multifractal-Julia (9 test functions)

### Performance
- Row-based → Pixel-based parallelization: 2-4x speedup on multi-core CPUs
- Full utilization of all CPU cores during fractal rendering
- Work-stealing scheduler ensures balanced load distribution

## [0.1.4] - 2026-01-19

### Added
- **Powerbrot Feature**: Generalized Mandelbrot with configurable power parameter
  - Power range: -10.0 to 10.0 (slider) with full f64 precision (text input)
  - Formula: z^power + c (power=2 is classic Mandelbrot)
  - Different powers create unique symmetries (3=tricorn, 4=quatric, etc.)
  - Negative powers produce convergent patterns
  - Fixed singularity issue with z₁ = c starting point
- **Profiling Command-Line Flag**: `--profiling` or `-p` to enable performance logs
  - Help flag: `--help` or `-h` shows usage information
  - `perf_log!` macro for conditional logging throughout codebase
  - Normal operation is silent; profiling only when explicitly enabled
- **Colormap Parser Utility**: `build_scripts/coolors_parser.py` for palette conversion
  - Converts coolors.co XML palettes to forma-fractalis JSON format
  - Generates proper "stops" format with nested "color" objects
  - Command-line tool for easy colormap creation
- **Electric Neon Colormap**: 5-color neon palette (cyan, hot pink, yellow, electric purple, cyan)
- **Automated Colormap Registration**: `define_builtin_colormaps!` macro system
  - Single line to register new colormaps
  - Automatically generates: constants, load function, builtin check, dropdown list
  - Eliminates manual updates to multiple functions
- **Export Performance Monitoring**: Detailed timing breakdown in export pipeline
  - Logs: setup, render, filter, metadata, I/O times separately
  - Displays percentage breakdown for bottleneck identification
- **Open Directory Button**: Quick file explorer access from export section
  - Cross-platform: Windows (explorer), macOS (open), Linux (xdg-open)
  - Opens selected export directory in native file manager
- **Benchmark Suite**: Comprehensive performance testing framework
  - `benches/fractal_bench.rs` tests all fractals at multiple resolutions
  - Tests 3 resolutions (SD/HD/FHD) × 5 iteration counts
  - 10 runs per test with avg/min/max timing
  - Established v0.1.3 performance baseline
- **CHANGELOG.md**: Professional version history following Keep a Changelog format
  - Migrated all release notes from README.md
  - Semantic versioning with GitHub comparison links

### Changed
- **Input Debouncing**: Text inputs now delayed 500ms to prevent typing lag
  - Buttons/sliders trigger immediate redraws
  - Text fields wait for typing to finish before redrawing
  - Eliminates UI lag during parameter entry
- **README.md**: Simplified releases section, links to CHANGELOG.md
- All performance/debug logs now respect `--profiling` flag
- Updated AGENTS.md with colormap creation workflow and macro usage

### Fixed
- Negative power parameters no longer diverge immediately (z₁ = c fix)
- Colormap parser generates correct JSON format
- Electric Neon colormap properly registered and appears in dropdown
- **Performance**: Power=2.0 (classic Mandelbrot) now uses optimized multiplication instead of powf()
  - Was 13-30x slower in initial Powerbrot implementation
  - Fixed with conditional: `if power == 2.0 { z * z } else { z.powf(power) }`
  - Custom powers still work correctly

### Documentation
- Created BENCHMARKS.md with v0.1.3 baseline results
- Added comprehensive GPU acceleration plan (v0.2.0.md)
- Updated AGENTS.md with simplified colormap registration steps
- Detailed redraw system documentation in main.rs
- GUI architecture refactoring notes added to v0.1.5.md

### Performance
- Benchmarks confirm NO regression from v0.1.3
- HD (1280×720) @ 1024 iterations: 8-42ms across all fractals
- All performance targets met or exceeded
- Typing lag eliminated (was main "slowness" perception)

## [0.1.3] - 2026-01-18

### Added
- **New Fractal**: Tippets Mandelbrot variation with unique order-of-operations distortion
- **New Colormap**: Twilight Garden (peachy, mint, teal, mauve palette)
- **Interactive Example**: mandelpath.rs for visualizing Mandelbrot iteration paths
  - Right-click to generate iteration sequences
  - Visual arrows show trajectory through complex plane
- Performance profiling instrumentation throughout rendering pipeline
  - Measures render time, buffer allocation, conversion, texture upload
  - Console output with timing breakdown (moving to `--profiling` flag in v0.1.4)

### Changed
- Standardized mouse controls: left-click only for zoom rectangle
- Right-click reserved for specialized examples
- Julia Set default view adjusted for better initial display
- Improved pointer state checking for better interaction

### Fixed
- Export filenames now correctly include both width and height
- Zero compilation warnings throughout codebase

### Removed
- **Mint Lavender** colormap (replaced by Twilight Garden)
- **Complete migration to FractalView**: Removed all backward compatibility layers
  - Deleted deprecated `fractal.rs` module
  - Removed `render_mandelbrot()` legacy function

### Performance
- Verified performance: 1280×720 @ 256 iterations renders in ~9-13ms (60-80 FPS)
- Confirmed rayon parallelization working correctly

## [0.1.2] - 2026-01-14

### Added
- **Multi-Fractal Support**: Three fractal types now available
  - Mandelbrot Set (classic)
  - Julia Set (interactive c_real/c_imag parameters)
  - Burning Ship
- **PNG Metadata Export**: All render settings embedded in PNG tEXt chunks
  - Fractal type, view coordinates, zoom level
  - Fractal parameters (Julia c values, etc.)
  - Complete colormap data
  - Color modulation settings
  - Export settings (filter, supersample, scale)
  - Enables exact reproduction of any exported image
- **New Colormap**: Frozen Amaranth (purple/pink gradient)
- Julia Set classic coordinate presets for quick discovery
- Trait-based fractal system for extensibility
- [PNG Metadata Reader Tool](https://github.com/ConociendoAlmasMenosHastiadas/png_meta_reader)

### Changed
- Fractal type shown in window title
- Export filenames now include fractal type
- Recommended supersampling increased to 8× for sharper exports
- Random classic Julia coordinates on fractal switch
- `FractalView` replaces `MandelbrotView` (backward compatible)

### Technical
- New `src/fractals/` module structure
- Fractal trait with Mandelbrot, Julia, and BurningShip implementations
- Parameter system for fractal-specific controls
- PNG crate integration for direct tEXt chunk control
- num-complex integration for cleaner complex number operations

### Removed
- Examples folder (demo code consolidated)

## [0.1.1] - 2026-01-12

### Added
- **Unified Rendering Pipeline**: Single codebase for preview and export
- **Professional Image Filtering**:
  - Lanczos3 filter (high-quality sharp resampling)
  - Gaussian filter (smooth anti-aliasing)
- **Supersampling Support**: Render at 2×-4× resolution, then downsample
- **Linear/Logarithmic Color Scaling**: Toggle for smoother gradients
- New `rendering_pipeline.rs` module
- New `filtering.rs` module with extensible filter architecture

### Changed
- Default supersample set to 2× for better quality exports
- Export now uses unified rendering pipeline
- Filtering applied to export only (preview stays fast)
- Clear UI controls for filter selection

### Quality of Life
- Export status indicator ("⏳ Exporting...")
- Smooth 60 FPS preview maintained

## [0.1.0] - 2026-01-11

### Added
- Interactive Mandelbrot set explorer with real-time rendering
- **10 Built-in Color Schemes**:
  - Default, Fire, Ocean, Grayscale, Rainbow
  - Academic, Coral Sunset, Olive Symmetry, Orchid Garden
- Interactive color editor with drag-and-drop color stops
- Save/load custom colormaps as JSON
- Click-and-drag zoom navigation with scroll wheel resize
- PNG export at scalable resolutions
- Parallel rendering with Rayon (multi-threaded)
- Period modulation for repeating color patterns
- Custom interior color for points inside the set
- Iteration control (10-10,000 iterations)
- Custom dimensions for preview resolution

### Technical
- Rust 2021 edition
- egui/eframe immediate mode GUI
- Rayon data parallelism
- image crate for processing
- png crate for encoding
- num-complex for fractal math
- serde/serde_json for serialization

### Default Settings
- Preview resolution: 1280×720 (16:9)
- Default iterations: 256
- Default export scale: 3.0 (produces 3840×2160)

---

## Version History Quick Reference

- **v0.1.4** (2026-01-19): Powerbrot, profiling flag, benchmark suite, CHANGELOG.md, colormap utilities
- **v0.1.3** (2026-01-18): Tippets Mandelbrot, mandelpath example, performance profiling
- **v0.1.2** (2026-01-14): Multi-fractal support, PNG metadata, Julia sets
- **v0.1.1** (2026-01-12): Unified pipeline, filtering, supersampling
- **v0.1.0** (2026-01-11): Initial release with Mandelbrot explorer

[Unreleased]: https://github.com/ConociendoAlmasMenosHastiadas/forma-fractalis/compare/v0.1.4...HEAD
[0.1.4]: https://github.com/ConociendoAlmasMenosHastiadas/forma-fractalis/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/ConociendoAlmasMenosHastiadas/forma-fractalis/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/ConociendoAlmasMenosHastiadas/forma-fractalis/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/ConociendoAlmasMenosHastiadas/forma-fractalis/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ConociendoAlmasMenosHastiadas/forma-fractalis/releases/tag/v0.1.0
