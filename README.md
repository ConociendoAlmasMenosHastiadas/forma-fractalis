# Mandelrust - Interactive Mandelbrot Set Explorer

An interactive Mandelbrot set explorer built in Rust with real-time rendering, advanced color mapping, and image export capabilities.  I should mention this project is like 95% vibes.  Its a recreation of an old project I made for a Java course back in college, but this time made in rust and using LLMs to include a bunch of features I wished I had but just never got around to using.  

The project is for-fun for those that want to take a look at the mandelbrot set in weird ways.  The initial version is just based around the mandelbrot set as the base-case.  If I stick with it I want to add weirder fractals in.  

This makes for cool wallpapers, banners, profile pics, etc.  I hope its fun and if you have cool ideas I can take them under advisement.  

> **NON-PROGRAMMERS**: Pre-built Windows executables are available in the [`builds/`](builds/) folder - just download the .zip file and run!

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Rust](https://img.shields.io/badge/rust-2021-orange)
![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue)

## Features

### Advanced Color Mapping
- **10 Built-in Color Schemes**: Default, Fire, Ocean, Grayscale, Rainbow, Academic, Mint Lavender, Coral Sunset, Olive Symmetry, Orchid Garden
- **Interactive Color Editor**: Create custom gradients with drag-and-drop color stops
- **Save/Load Custom Colormaps**: Persist your color schemes as JSON files
- **Live Preview**: Real-time gradient visualization
- **Color Stop Management**: Add, edit, delete, and position color stops with precision
- **Named Colors**: Optional color names for documentation (e.g., "Celadon", "Tiffany Blue")

### Interactive Navigation
- **Click-and-Drag Zoom**: Position a zoom box and click to zoom in
- **Mouse Wheel Control**: Dynamically adjust zoom box size
- **Reset View**: Quick return to default view
- **Precision Controls**: Manual coordinate and zoom input

### High-Quality Export
- **PNG Export**: Lossless image output
- **Scalable Resolution**: 3x default (3840×2160 from 1280×720 preview)
- **Custom Output Directory**: Choose where to save your renders
- **Timestamped Filenames**: Automatic file naming

### Performance
- **Parallel Rendering**: Multi-threaded computation using Rayon
- **Real-time Updates**: Smooth 60 FPS interface with egui
- **Efficient Color Mapping**: Optimized gradient interpolation
- **Smooth Coloring**: Logarithmic scaling for better color distribution

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
- Download `mandelrust_v0.1.0_windows.zip`
- Extract and run `mandelrust.exe`
- No compilation required!

### Build from Source

```bash
# Clone the repository
git clone https://github.com/ConociendoAlmasMenosHastiadas/mandelrust.git
cd mandelrust

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
   - **Pre-built**: Extract and run `mandelrust.exe`
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
mandelrust/
├── src/
│   ├── main.rs              # Application entry point & GUI state
│   ├── lib.rs               # Library exports
│   ├── fractal.rs           # Mandelbrot mathematics
│   ├── rendering.rs         # Parallel rendering engine
│   ├── colorschemes.rs      # Color types & gradient system
│   ├── colorschemes_io.rs   # Save/load functionality
│   ├── colorschemes_gui.rs  # Color editor widgets
│   ├── gui.rs               # UI sections & helpers
│   ├── export.rs            # PNG export functionality
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
│       └── orchid_garden.json
└── examples/
    ├── colormap_io.rs       # ColorMap save/load demo
    └── colormap_names.rs    # Color names demo
```

### Key Technologies

- **egui/eframe**: Immediate mode GUI framework
- **Rayon**: Data parallelism for rendering
- **image**: PNG encoding/decoding
- **serde/serde_json**: ColorMap serialization
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

Run example programs to see ColorMap features:

```bash
# Demonstrate save/load functionality
cargo run --example colormap_io

# Show color names in colormaps
cargo run --example colormap_names
```

## Configuration

Default preview resolution: 1280×720 (16:9)
Default iterations: 256
Default export scale: 3.0 (produces 3840×2160)

Customize in the GUI or modify default values in `src/main.rs`

## Contributing

Contributions welcome! Areas for improvement:
- Additional fractal types (Julia sets, Burning Ship, etc.)
- More color interpolation modes (HSV, LAB)
- Animation/zoom sequence export
- Saved location bookmarks
- Undo/redo for color editing

## License

Dual-licensed under Apache-2.0 or MIT. See LICENSE-APACHE and LICENSE-MIT files for details.

## Third-Party Licenses

See [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for a comprehensive list of all dependency licenses.

## Credits

Developed using Rust 2021 edition with egui framework.
Various Palettes created using Coolors.co — thanks Coolors!