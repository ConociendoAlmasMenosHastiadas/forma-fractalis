# Forma Fractalis - Interactive Fractal Explorer

An interactive fractal explorer built in Rust with real-time rendering, advanced color mapping, and image export capabilities. Explore the Mandelbrot set, Julia sets, and Burning Ship fractal with a modern, high-performance interface. I should mention this project is like 95% vibes. It's a recreation of an old project I made for a Java course back in college, but this time made in Rust and using LLMs to include a bunch of features I wished I had but just never got around to making.

The project is for-fun for those that want to explore fractals in weird ways. Now featuring multiple fractal types with more to come!  

This makes for cool wallpapers, banners, profile pics, etc.  I hope its fun and if you have cool ideas I can take them under advisement.  

> **NON-PROGRAMMERS**: Pre-built Windows executables are available in the [`builds/`](builds/) folder - just download the .zip file and run!

![Version](https://img.shields.io/badge/version-0.1.7-blue)
![Rust](https://img.shields.io/badge/rust-2021-orange)
![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue)

## Features

### Command-Line Rendering (New in v0.1.7!)
- **Headless Rendering**: Generate fractals without GUI
- **Load from PNG or JSON**: Use exported settings from GUI or standalone JSON files
- **Parameter Overrides**: Change width, height, iterations, scale, and supersample via CLI
- **Progress Indicators**: Real-time rendering progress with elapsed time
- **Batch Processing**: Render multiple resolutions from same settings
- **Automation Ready**: Perfect for CI/CD, server rendering, or scripting
- **Export Settings**: Save fractal configuration as JSON for CLI use

### Multi-Fractal Support
- **Six Fractal Types**: Mandelbrot Set, Julia Set, Burning Ship, Tippets Mandelbrot, Multifractal-Julia, and Cactus
- **Interactive Fractal Selector**: Switch between fractals instantly
- **Fractal-Specific Parameters**: 
  - Julia Set: Adjustable c_real and c_imag parameters with real-time sliders
  - Multifractal-Julia: Power parameter k (-5.0 to 5.0) with cycle detection
  - Cactus: Unique cubic iteration with adaptive escape radius
  - Classic Julia coordinates presets for quick discovery
- **Optimized Default Views**: Each fractal loads with ideal starting position and zoom
- **Trait-Based Architecture**: Extensible framework for adding more fractals
- **Unique Features**: Multifractal-Julia includes period detection for cyclic orbits

### Advanced Color Mapping
- **Powered by [scala-chromatica](https://github.com/ConociendoAlmasMenosHastiadas/scala-chromatica)**: Framework-agnostic color gradient library
- **14 Built-in Color Schemes**: Default, Fire, Ocean, Grayscale, Rainbow, Academic, Coral Sunset, Olive Symmetry, Orchid Garden, Frozen Amaranth, Twilight Garden, Electric Neon, Cosmic Dawn, Vintage Lavender
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
- **Load from PNG**: Import fractal settings from previously exported images
- **Complete Round-Trip**: Export → Load → Exact reproduction of any fractal
- **Scalable Resolution**: 3x default (3840×2160 from 1280×720 preview)
- **Image Filtering**: Lanczos3 & Gaussian filters for professional quality
- **Supersampling**: Render at 8x resolution (recommended), then downsample for ultra-sharp results
- **PNG Metadata Embedding**: All render settings saved in PNG tEXt chunks
  - Fractal type (Mandelbrot, Julia Set, Burning Ship, Tippets Mandelbrot, Multifractal-Julia)
  - View coordinates (center_x, center_y, zoom level)
  - Fractal parameters (e.g., Julia c values, powers)
  - Complete colormap data (name and full color stops)
  - Color modulation settings (period, interior color, log scale)
  - Export settings (filter type, supersample, scale)
  - Creation timestamp for organization
  - **Reproducible renders**: Load any exported PNG and recreate the exact same image
  - **Documentation**: See [METADATA_FORMAT.md](METADATA_FORMAT.md) for complete format reference
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

# Run the application (GUI mode)
cargo run --release

# Or use CLI mode for headless rendering
cargo run --release -- render --input settings.json --output fractal.png
```

### Create Distribution Package (Windows)

```powershell
# Run the build script to create a distribution zip
.\build_scripts\windows-build.ps1

# Output will be in builds/ folder
```

## Quick Start

### GUI Mode
1. **Launch the application**: 
   - **Pre-built**: Extract and run `forma-fractalis.exe`
   - **From source**: Run `cargo run --release`
2. **Navigate**: Click and drag to position zoom box, scroll to resize, click to zoom
3. **Change Colors**: Select a color scheme from the dropdown
4. **Export**: Set scale (default 3.0), choose directory, click "Export PNG"

### CLI Mode (New in v0.1.7)
```bash
# Render from PNG with embedded metadata
forma-fractalis render --input input.png --output output.png

# Render from JSON settings file
forma-fractalis render --input settings.json --output output.png

# Override parameters
forma-fractalis render --input input.png --output output.png --width 3840 --height 2160 --iterations 1024

# Scale and supersample
forma-fractalis render --input input.png --output output.png --scale 3.0 --supersample 8

# Get help
forma-fractalis --help
forma-fractalis render --help
```

## Usage Guide

### GUI Navigation
- **Position Zoom Box**: Click and drag on the fractal
- **Resize Zoom Box**: Scroll mouse wheel while dragging
- **Apply Zoom**: Click again to zoom in
- **Reset View**: Use the "Reset View" button in the sidebar

### Color Customization

#### Using Built-in Schemes
1. Open the "Color Scheme" dropdown
2. Select from 14 pre-built colormaps

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

#### From GUI
1. **Set Scale**: Enter scale factor (3.0 = 3840×2160 output)
2. **Choose Directory** (optional): Click "Choose Directory"
3. **Export PNG**: Renders image with all current settings
4. **Export Settings (JSON)**: Saves settings without rendering (for CLI use)

#### From CLI
```bash
# Basic render from PNG
forma-fractalis render -i input.png -o output.png

# Render from JSON with overrides
forma-fractalis render -i settings.json -o output.png --width 7680 --height 4320

# High-quality 8K render
forma-fractalis render -i input.png -o wallpaper.png --scale 6.0 --supersample 8
```

### Advanced Settings

#### Period Modulation
- Enable "Period" checkbox
- Adjust value to create repeating color cycles
- Use ×2/÷2 buttons for quick adjustments

#### Interior Color
- Enable "Interior Color" checkbox
- Click color box to open picker
- Set custom color for points inside the set (usually black)

## Command-Line Interface

### Render Command

**Usage**: `forma-fractalis render [OPTIONS] --input <FILE> --output <FILE>`

**Options**:
- `-i, --input <FILE>` - Input file (PNG with metadata or JSON settings)
- `-o, --output <FILE>` - Output PNG file path
- `--width <WIDTH>` - Override width (in pixels)
- `--height <HEIGHT>` - Override height (in pixels)
- `--iterations <ITERATIONS>` - Override max iterations
- `--scale <SCALE>` - Override scaling factor (e.g., 3.0 for 3x size)
- `--supersample <SUPERSAMPLE>` - Override supersample multiplier (1-16)

**Examples**:
```bash
# Reproduce an exact render
forma-fractalis render -i mandelbrot_3840x2160.png -o reproduction.png

# Change resolution
forma-fractalis render -i input.png -o output.png --width 1920 --height 1080

# High-detail render
forma-fractalis render -i input.json -o detailed.png --iterations 2048 --supersample 8

# Quick test render
forma-fractalis render -i input.png -o test.png --scale 1.0 --supersample 1
```

### Workflow: GUI → CLI Pipeline

1. **Design in GUI**: Explore, adjust colors, find interesting regions
2. **Export Settings**: Click "Export Settings (JSON)" button
3. **Batch Render**: Use CLI to render at different resolutions/settings
4. **Share**: JSON files are small and version-control friendly

```bash
# Create settings in GUI, then:
forma-fractalis render -i my_fractal.json -o desktop_1920x1080.png --width 1920 --height 1080
forma-fractalis render -i my_fractal.json -o desktop_3840x2160.png --width 3840 --height 2160
forma-fractalis render -i my_fractal.json -o poster_7680x4320.png --width 7680 --height 4320
```

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
│   │   ├── burning_ship.rs  # Burning Ship
│   │   └── tippets_mandelbrot.rs  # Tippets Mandelbrot
│   └── colormaps/           # Built-in colormap JSON files
│       ├── default.json
│       ├── fire.json
│       ├── ocean.json
│       ├── grayscale.json
│       ├── rainbow.json
│       ├── academic.json
│       ├── coral_sunset.json
│       ├── olive_symmetry.json
│       ├── orchid_garden.json
│       ├── frozen_amaranth.json
│       └── twilight_garden.json
└── examples/
    ├── mandelpath.rs        # Iteration path visualizer
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

## Third-Party Licenses

See [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for a comprehensive list of all dependency licenses.

## Releases

For detailed release notes and version history, see [CHANGELOG.md](CHANGELOG.md).

### Latest: v0.1.7 (February 14, 2026)
- **Command-Line Rendering**: Headless fractal generation from PNG or JSON files
- **Parameter Overrides**: CLI flags for width, height, iterations, scale, supersample
- **Progress Indicators**: Real-time rendering progress with indicatif library
- **Export Settings as JSON**: Save fractal configuration for CLI use (standalone JSON)
- **Marek Dragon Fractal**: New rotation-based fractal (z_{n+1} = exp(jφ)z_n + z_n²)
- **State Conversion System**: Bidirectional From trait implementations (70% line reduction)
- **Refactored Export**: New export_png_from_state() simplifies call sites
- **Simplified Load**: load_from_metadata() reduced from 50+ lines to 15
- **scala-chromatica Migration Complete**: Removed deprecated bridge modules
- **44 Tests Passing**: 7 new MarekDragon unit tests added
- **Foundation for Automation**: Clean state management enables future scripting features

### Previous: v0.1.61 (February 5, 2026)
- **Colormap Library Extraction**: Colormaps moved to standalone [scala-chromatica](https://github.com/ConociendoAlmasMenosHastiadas/scala-chromatica) crate
- **Framework-Agnostic**: Reusable color gradient library for any Rust project
- **Zero Functional Changes**: Pure refactoring for code reuse
- **Deprecation Path**: Bridge modules for smooth transition (removed in v0.1.7)
- **All Tests Passing**: 35 tests confirm backwards compatibility

### Previous: v0.1.6 (January 26, 2026)
- **Load from PNG**: Import fractal settings from exported images - complete round-trip functionality
- **Cactus Fractal**: New cubic iteration fractal (z_{n+1} = z_n^3 + (z_0 - 1)z_n - z_0)
- **Vintage Lavender Colormap**: Muted earth tones palette (lavender, teal, slate, tan, pumpkin)
- **Enhanced Metadata**: Added timestamp field for organization and tracking
- **PNG Metadata Documentation**: New [METADATA_FORMAT.md](METADATA_FORMAT.md) with complete format reference
- **Better Error Handling**: Graceful errors for invalid/missing metadata
- **Foundation for CLI**: Metadata structure ready for command-line rendering (v0.1.7)

### Previous: v0.1.5 (January 19, 2026)
- **Multifractal-Julia Fractal**: New fractal with cycle detection (z_{n+1} = c^k · z_n^{-2} + c)
- **Cosmic Dawn Colormap**: Deep space to dawn gradient (Prussian Blue → Cotton Rose)
- **Pixel-Level Parallelization**: 2-4x rendering speedup, 75-100% CPU utilization
- **Centralized Constants**: number_utils module for consistent precision
- **Performance**: Full multi-core utilization with improved work distribution

### v0.1.4 (January 19, 2026)
- Powerbrot: Mandelbrot with configurable power (-10.0 to 10.0)
- Profiling command-line flag (--profiling) for performance logs
- Benchmark suite with v0.1.3 baseline
- CHANGELOG.md for professional version tracking
- Colormap parser utility for coolors.co palettes
- Electric Neon colormap
- Input debouncing (fixes typing lag)
- Export performance monitoring
- Open Directory button

## Credits

Developed using Rust 2021 edition with egui framework.
Various Palettes created using Coolors.co — thanks Coolors!