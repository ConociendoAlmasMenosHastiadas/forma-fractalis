# Forma Fractalis - Interactive Fractal Explorer

An interactive fractal explorer built in Rust with real-time rendering, advanced color mapping, and image export capabilities. Explore the Mandelbrot set, Julia sets, and Burning Ship fractal with a modern, high-performance interface. I should mention this project is like 95% vibes. It's a recreation of an old project I made for a Java course back in college, but this time made in Rust and using LLMs to include a bunch of features I wished I had but just never got around to making.

The project is for-fun for those that want to explore fractals in weird ways. Now featuring multiple fractal types with more to come!  

This makes for cool wallpapers, banners, profile pics, etc.  I hope its fun and if you have cool ideas I can take them under advisement.  

> **NON-PROGRAMMERS**: Pre-built Windows executables are available in the [`builds/`](builds/) folder - just download the .zip file and run!

![Version](https://img.shields.io/badge/version-0.1.3-blue)
![Rust](https://img.shields.io/badge/rust-2021-orange)
![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue)

## Features

### Multi-Fractal Support
- **Four Fractal Types**: Mandelbrot Set, Julia Set, Burning Ship, and Tippets Mandelbrot
- **Interactive Fractal Selector**: Switch between fractals instantly
- **Fractal-Specific Parameters**: 
  - Julia Set: Adjustable c_real and c_imag parameters with real-time sliders
  - Classic Julia coordinates presets for quick discovery
- **Optimized Default Views**: Each fractal loads with ideal starting position and zoom
- **Trait-Based Architecture**: Extensible framework for adding more fractals

### Advanced Color Mapping
- **11 Built-in Color Schemes**: Default, Fire, Ocean, Grayscale, Rainbow, Academic, Coral Sunset, Olive Symmetry, Orchid Garden, Frozen Amaranth, Twilight Garden
- **Interactive Color Editor**: Create custom gradients with drag-and-drop color stops
- **Save/Load Custom Colormaps**: Persist your color schemes as JSON files
- **Live Preview**: Real-time gradient visualization
- **Color Stop Management**: Add, edit, delete, and position color stops with precision
- **Named Colors**: Optional color names for documentation (e.g., "Celadon", "Tiffany Blue")
- **Linear & Logarithmic Scaling**: Toggle between color distribution modes

### Interactive Navigation
- **Click-and-Drag Zoom**: Position a zoom box with left mouse button and scroll wheel to resize
- **Standardized Mouse Controls**: Left-click for zoom, right-click reserved for specialized examples
- **Reset View**: Quick return to default view
- **Precision Controls**: Manual coordinate and zoom input

### High-Quality Export
- **PNG Export**: Lossless image output
- **Scalable Resolution**: 3x default (3840×2160 from 1280×720 preview)
- **Image Filtering**: Lanczos3 & Gaussian filters for professional quality
- **Supersampling**: Render at 8x resolution (recommended), then downsample for ultra-sharp results
- **PNG Metadata Embedding**: All render settings saved in PNG tEXt chunks
  - Fractal type (Mandelbrot, Julia Set, Burning Ship)
  - View coordinates (center_x, center_y, zoom level)
  - Fractal parameters (e.g., Julia c values)
  - Complete colormap data (name and full color stops)
  - Color modulation settings (period, interior color, log scale)
  - Export settings (filter type, supersample, scale)
  - **Reproducible renders**: Load any exported PNG and recreate the exact same image
  - **Metadata reader available**: [PNG Meta Reader](https://github.com/ConociendoAlmasMenosHastiadas/png_meta_reader)
- **Custom Output Directory**: Choose where to save your renders
- **Timestamped Filenames**: Automatic file naming with fractal type

### Performance
- **Parallel Rendering**: Multi-threaded computation using Rayon
- **Real-time Updates**: Smooth 60 FPS interface with egui
- **Efficient Color Mapping**: Optimized gradient interpolation
- **Unified Pipeline**: Consistent behavior between preview and export
- **Smooth Coloring**: Optional logarithmic scaling for better color distribution
- **Performance Profiling**: Built-in timing instrumentation for debugging (console output)

### Advanced Options
- **Period Modulation**: Create repeating color patterns
- **Custom Interior Color**: Set color for points inside the Mandelbrot set
- **Iteration Control**: Adjust detail level (10-10,000 iterations)
- **Custom Dimensions**: Configurable preview resolution

## Installation

### Prerequisites
- **Rust**: 1.70 or later (2021 edition)
- **Cargo**: Latest stable version

### Download Pre-built Binary (Windows)

Ready-to-use Windows distributions are available in the [`builds/`](builds/) folder:
- Download `forma-fractalis_v0.1.3_windows.zip`
- Extract and run `forma-fractalis.exe`
- No compilation required!

### Build from Source

```bash
# Clone the repository
git clone https://github.com/ConociendoAlmasMenosHastiadas/forma-fractalis.git
cd forma-fractalis

# Build release version
cargo build --release

# Run the application
cargo run --release
```

### Create Distribution Package (Windows)

```powershell
# Run the build script to create a distribution zip
.\build_scripts\windows-build.ps1

# Output will be in builds/ folder
```

## Quick Start

1. **Launch the application**: 
   - **Pre-built**: Extract and run `forma-fractalis.exe`
   - **From source**: Run `cargo run --release`
2. **Navigate**: Click and drag to position zoom box, scroll to resize, click to zoom
3. **Change Colors**: Select a color scheme from the dropdown
4. **Export**: Set scale (default 3.0), choose directory, click "Export PNG"

## Usage Guide

### Navigation
- **Position Zoom Box**: Click and drag on the fractal
- **Resize Zoom Box**: Scroll mouse wheel while dragging
- **Apply Zoom**: Click again to zoom in
- **Reset View**: Use the "Reset View" button in the sidebar

### Color Customization

#### Using Built-in Schemes
1. Open the "Color Scheme" dropdown
2. Select from 10 pre-built colormaps

#### Creating Custom Colormaps
1. Open the "Save/Load ColorMap" collapsible section
2. Drag color stops on the gradient preview
3. Click stops to edit RGB values
4. Add new stops with "Add Color Stop" button
5. Click "Save ColorMap" to save as JSON

#### Loading Custom Colormaps
1. Click "Choose Directory" (optional - sets working folder)
2. Click "Load ColorMap"
3. Select a JSON file from the file picker

### Exporting Images

1. **Set Scale**: Enter scale factor (3.0 = 3840×2160 output)
2. **Choose Directory** (optional): Click "Choose Directory"
3. **Export**: Click "Export PNG"
4. Files are saved with format: `mandelbrot_WIDTHxHEIGHT_TIMESTAMP.png`

### Advanced Settings

#### Period Modulation
- Enable "Period" checkbox
- Adjust value to create repeating color cycles
- Use ×2/÷2 buttons for quick adjustments

#### Interior Color
- Enable "Interior Color" checkbox
- Click color box to open picker
- Set custom color for points inside the set (usually black)

## Architecture

### Module Structure

```
forma-fractalis/
├── src/
│   ├── main.rs              # Application entry point & GUI state
│   ├── lib.rs               # Library exports
│   ├── rendering.rs         # Parallel rendering engine
│   ├── rendering_pipeline.rs # Unified rendering pipeline
│   ├── filtering.rs         # Image filtering algorithms
│   ├── colorschemes.rs      # Color types & gradient system
│   ├── colorschemes_io.rs   # Save/load functionality
│   ├── colorschemes_gui.rs  # Color editor widgets
│   ├── gui.rs               # UI sections & helpers
│   ├── export.rs            # PNG export with metadata
│   ├── fractals/            # Trait-based fractal system
│   │   ├── mod.rs           # Fractal trait & FractalView
│   │   ├── mandelbrot.rs    # Mandelbrot Set
│   │   ├── julia.rs         # Julia Set
│   │   └── burning_ship.rs  # Burning Ship
│   └── colormaps/           # Built-in colormap JSON files
│       ├── default.json
│       ├── fire.json
│       ├── ocean.json
│       ├── grayscale.json
│       ├── rainbow.json
│       ├── academic.json
│       ├── mint_lavender.json
│       ├── coral_sunset.json
│       ├── olive_symmetry.json
│       ├── orchid_garden.json
│       └── frozen_amaranth.json
└── examples/
    ├── colormap_io.rs       # ColorMap save/load demo
    └── colormap_names.rs    # Color names demo
```

### Key Technologies

- **egui/eframe**: Immediate mode GUI framework
- **Rayon**: Data parallelism for rendering
- **image**: Image processing and filtering
- **png**: Direct PNG encoding with metadata support
- **num-complex**: Complex number operations for fractals
- **serde/serde_json**: ColorMap and metadata serialization
- **directories**: Platform-specific config paths
- **rfd**: Native file dialogs

## ColorMap System

See [COLORMAP_SAVELOAD.md](COLORMAP_SAVELOAD.md) for detailed documentation on:
- Built-in vs custom colormaps
- JSON file format with color names
- Programmatic API usage
- Platform-specific storage locations

## Performance Tips

- **Lower iterations** for faster preview (256-512)
- **Higher iterations** for final export (1000+)
- **Parallel rendering** automatically uses all CPU cores
- **Period modulation** can create interesting effects at lower iteration counts

## Examples

Explore specialized fractal visualization tools:

```bash
# Visualize Mandelbrot iteration paths
cargo run --example mandelpath

# Demonstrate save/load functionality
cargo run --example colormap_io

# Show color names in colormaps
cargo run --example colormap_names
```

**mandelpath**: Interactive tool to visualize how points iterate in the Mandelbrot set. Right-click to generate iteration paths with visual arrows showing the trajectory.

## Configuration

Default preview resolution: 1280×720 (16:9)
Default iterations: 256
Default export scale: 3.0 (produces 3840×2160)

Customize in the GUI or modify default values in `src/main.rs`

## Contributing

Contributions welcome! Areas for improvement:
- Additional fractal types (Tertation, Newton, etc.)
- More color interpolation modes (HSV, LAB)
- Animation/zoom sequence export
- Load settings from PNG metadata ("Load from PNG" feature)
- Saved location bookmarks
- Undo/redo for color editing

## License

Dual-licensed under Apache-2.0 or MIT. See LICENSE-APACHE and LICENSE-MIT files for details.

## Third-Party Licens8, 2026)

**New Fractals:**
- **Tippets Mandelbrot**: A variation with reciprocal term (z² + c + 1/z) creating unique distortions

**New Colormaps:**
- **Twilight Garden**: Soft, natural palette with peachy tones, mint greens, deep teals, and mauve
- Removed: Mint Lavender (replaced by Twilight Garden)

**Examples & Tools:**
- **mandelpath.rs**: Interactive visualization of Mandelbrot iteration paths
  - Right-click to generate iteration sequences
  - Visual arrows show trajectory through complex plane
  - Left-click drag for zoom, scroll wheel for zoom box size

**Mouse Interaction Improvements:**
- Standardized mouse controls: left-click only triggers zoom rectangle
- Right-click reserved for future features/specialized examples
- Improved pointer state checking for better interaction

**Performance Analysis:**
- Added performance profiling instrumentation
  - Measures render time, buffer allocation, image conversion, texture upload
  - Console output shows timing breakdown
  - Verified performance: 1280x720 @ 256 iterations renders in ~9-13ms (60-80 FPS)
  - Confirmed rayon parallelization working correctly
- Note: Profiling output planned to move behind `--profiling` flag in v0.1.4

**Code Cleanup & Architecture:**
- **Complete Migration to FractalView**: Removed all backward compatibility layers
  - Deleted deprecated `fractal.rs` module
  - FractalView now used consistently throughout codebase
  - Removed `render_mandelbrot()` legacy function
- **Bug Fixes**:
  - Export filenames now correctly include both width and height
  - Julia Set default view adjusted for better initial display
- **Code Quality**:
  - Added helper methods to FractalType enum and FractalView
  - Cleaner, more maintainable codebase
  - Zero compilation warnings
  - Updated documentation

**Files Modified**: 16 files across core modules, examples, and planning documentsn to reflect multi-fractal architecture

**Technical:**
- All deprecated code paths removed
- FractalView fully replaces MandelbrotView across all modules
- Compilation produces zero warnings
- Test suite updated and passing

### v0.1.2 (January 14, 2026)

**Major Features:**
- **Multi-Fractal Support**: Explore three fractal types:
  - **Mandelbrot Set**: Classic fractal with deep zoom capability
  - **Julia Set**: Interactive parameters (c_real, c_imag) with real-time sliders and curated classic coordinates
  - **Burning Ship**: Unique fractal with ship-like structures
- **PNG Metadata Export**: All render settings embedded in PNG tEXt chunks
  - Fractal type, view coordinates, zoom level
  - Fractal parameters (Julia c values, etc.)
  - Complete colormap data for exact reproduction
  - Color modulation settings (period, interior color, log scale)
  - Export settings (filter, supersample, scale)
  - Future-ready for "Load from PNG" feature
  - [Metadata Reader Tool](https://github.com/ConociendoAlmasMenosHastiadas/png_meta_reader) available
- **New Color Scheme**: Frozen Amaranth - beautiful purple/pink gradient

**Architecture:**
- **Trait-Based Fractal System**: Extensible framework with `Fractal` trait
- **num-complex Integration**: Cleaner complex number operations
- **Dynamic GUI**: Parameter controls adapt to selected fractal type
- **Unified Export Pipeline**: Metadata embedded seamlessly in PNG export

**Quality of Life:**
- Recommended supersampling increased to 8x for sharper exports
- Fractal type shown in window title
- Export filenames include fractal type
- Random classic Julia coordinates on fractal switch
- Removed examples folder (demo code consolidated)

**Technical:**
- New `src/fractals/` module with Mandelbrot, Julia, and BurningShip implementations
- `FractalView` replaces `MandelbrotView` (backward compatibility maintained)
- Parameter system for fractal-specific controls
- PNG crate integration for direct tEXt chunk control

### v0.1.1 (January 12, 2026)

**New Features:**
- **Unified Rendering Pipeline**: Preview and export now use the same rendering codebase, eliminating duplication
- **Professional Image Filtering**: Added Lanczos3 and Gaussian filters for export
- **Supersampling Support**: Render at 2x-4x resolution, then downsample for ultra-sharp results
- **Linear/Logarithmic Color Scaling**: Toggle between color distribution modes for smoother gradients

**Architecture Improvements:**
- New `rendering_pipeline.rs` module with unified rendering system
- New `filtering.rs` module with extensible filter architecture
- Refactored export system to use unified pipeline
- Easy to extend with additional filters in the future

**Quality of Life:**
- Export-only filtering keeps preview fast and responsive
- Default supersample set to 2x for better quality exports
- Clear UI controls for filter selection
- Export status indicator ("⏳ Exporting...")

**Breaking Changes:** None - fully backward compatible. Default behavior (no filtering) matches v0.1.0.

### v0.1.0 (Initial Release)

**Core Features:**
- Interactive Mandelbrot set explorer with real-time rendering
- 10 built-in color schemes
- Interactive color editor with drag-and-drop color stops
- Save/load custom colormaps as JSON
- Click-and-drag zoom navigation
- PNG export at scalable resolutions
- Parallel rendering with Rayon
- Period modulation and custom interior colors

## Credits

Developed using Rust 2021 edition with egui framework.
Various Palettes created using Coolors.co — thanks Coolors!