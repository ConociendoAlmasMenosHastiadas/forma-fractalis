# Changelog

All notable changes to Forma Fractalis will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.61] - 2026-02-05

### Changed
- **Colormap Library Extraction**: Colormaps moved to standalone `scala-chromatica` crate
  - Core colormap functionality now in separate, reusable crate
  - 14 built-in colormaps (Fire, Ocean, Rainbow, Academic, etc.)
  - Smooth RGB interpolation and HSV color space support
  - Platform-specific config directory management
  - JSON serialization for custom colormaps
  - Available at: https://github.com/ConociendoAlmasMenosHastiadas/scala-chromatica

### Deprecated
- `src/colorschemes.rs` - Use `scala_chromatica` crate directly (removed in v0.1.7)
- `src/colorschemes_io.rs` - Use `scala_chromatica::io` module (removed in v0.1.7)
- `src/colormaps/*.json` - 14 JSON files now in scala-chromatica (removed in v0.1.7)

### Added
- Dependency on `scala-chromatica` crate via git
- Bridge modules with deprecation warnings for smooth transition
- `ColorScheme` enum remains for GUI-specific logic

### Technical
- All imports updated to use `scala_chromatica` directly
- GUI functionality (`colorschemes_gui.rs`) remains in forma-fractalis
- Zero functional changes - pure refactoring for code reuse
- All tests passing (35 tests)
- Deprecation warnings guide users to new API

### Migration Guide
```rust
// Before (deprecated)
use forma_fractalis::colorschemes::{Color, ColorMap};
use forma_fractalis::colorschemes_io;

// After
use scala_chromatica::{Color, ColorMap};
use scala_chromatica::io;
```

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
- Error messages use ✓ and ❌ emojis for visual feedback
- Status message prefixing ("LOAD_PNG:") for GUI-to-logic communication
- Cactus fractal uses adaptive escape radius for accuracy

### Testing
- ✅ All 28 unit tests passing (7 new for Cactus fractal)
- ✅ Code compiles without errors or warnings
- ✅ Release build successful
- ✅ Manual testing confirmed: export → load → exact reproduction
- ✅ Cactus fractal renders correctly with Vintage Lavender colormap

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
