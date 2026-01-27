# ColorMap Save/Load System

## Overview

The ColorMap save/load system provides persistent storage for color schemes using JSON files. It supports both built-in default colormaps (embedded in the binary) and user-created custom colormaps that can be saved anywhere on the filesystem.

## Architecture

### Built-in ColorMaps
- **Embedded at compile time** using `include_str!` macro
- Always available, cannot be lost or corrupted
- No runtime file I/O needed for defaults
- Located in: `src/colormaps/*.json`
- Available via dropdown in GUI

### Custom ColorMaps
- **User-controlled location**: Save/load from any directory via file dialogs
- **File picker integration**: Native OS dialogs for intuitive file management
- **JSON format**: Human-readable and editable
- **Named color support**: Optional color names in stops (e.g., "Celadon", "Tiffany Blue")

## UI Integration

### Accessing Save/Load

In the GUI, open the **"💾 Save/Load ColorMap"** collapsible section in the color editor sidebar.

### Saving a ColorMap

1. **Choose Directory** (optional): Click "📁 Choose Directory" to set a working folder
2. **Create/Edit**: Design your colormap using the gradient editor
3. **Save**: Click "💾 Save ColorMap"
4. **Name**: File dialog opens - enter a filename
5. The colormap is saved with metadata including color names

### Loading a ColorMap

1. **Choose Directory** (optional): Click "📁 Choose Directory" to browse a specific folder
2. **Load**: Click "📂 Load ColorMap"  
3. **Select**: File picker opens - choose a `.json` file
4. The colormap is loaded and applied immediately

## JSON Format

ColorMaps are stored as JSON files with the following structure:

```json
{
  "name": "Fire",
  "stops": [
    {
      "position": 0.0,
      "color": {
        "r": 0,
        "g": 0,
        "b": 0
      }
    },
    {
      "position": 0.5,
      "color": {
        "r": 255,
        "g": 0,
        "b": 0
      }
    },
    {
      "position": 1.0,
      "color": {
        "r": 255,
        "g": 255,
        "b": 255
      }
    }
  ]
}
```

## Usage

### Loading Built-in ColorMaps

Built-in colormaps are available via the **Color Scheme** dropdown in the main GUI. Select from:
- Default, Fire, Ocean, Grayscale, Rainbow
- Academic, Mint Lavender, Coral Sunset, Olive Symmetry, Orchid Garden

### Programmatic API

For scripting or automation, use the `colorschemes_io` module:

```rust
use forma_fractalis::colorschemes_io::load_builtin_colormap;

let fire = load_builtin_colormap("Fire")?;
```

Available built-in colormaps: `Default`, `Fire`, `Ocean`, `Grayscale`, `Rainbow`

### Loading Any ColorMap

```rust
use forma_fractalis::colorschemes_io::load_colormap;

// Will check built-ins first, then custom colormaps
let colormap = load_colormap("MyCustom")?;
```

### Saving Custom ColorMaps

```rust
use forma_fractalis::colorschemes::{ColorMap, ColorStop, Color};
use forma_fractalis::colorschemes_io::save_colormap;

let custom = ColorMap::new(
    "MyCustom".to_string(),
    vec![
        ColorStop::new(0.0, Color::black()),
        ColorStop::new(1.0, Color::white()),
    ],
);

let path = save_colormap(&custom)?;
println!("Saved to: {}", path.display());
```

### Listing Available ColorMaps

```rust
use forma_fractalis::colorschemes_io::list_available_colormaps;

let colormaps = list_available_colormaps()?;
for info in colormaps {
    println!("{} - {}", 
        info.name, 
        if info.is_builtin { "built-in" } else { "custom" }
    );
}
```

### Deleting Custom ColorMaps

```rust
use forma_fractalis::colorschemes_io::delete_custom_colormap;

delete_custom_colormap("MyCustom")?;
```

Note: Built-in colormaps cannot be deleted.

### Exporting Built-in ColorMaps

To create a customized version of a built-in colormap:

```rust
use forma_fractalis::colorschemes_io::export_builtin_colormap;

// This copies the built-in to the custom directory where you can edit it
let path = export_builtin_colormap("Ocean")?;
```

## API Reference

### Functions

#### `get_colormaps_directory() -> Result<PathBuf>`
Returns the platform-specific directory where custom colormaps are stored.

#### `load_builtin_colormap(name: &str) -> Result<ColorMap>`
Loads a built-in colormap by name. Returns error if name doesn't match a built-in.

#### `is_builtin_colormap(name: &str) -> bool`
Checks if a name corresponds to a built-in colormap.

#### `save_colormap(colormap: &ColorMap) -> Result<PathBuf>`
Saves a colormap to the custom directory. Returns the path where it was saved.

#### `load_custom_colormap(name: &str) -> Result<ColorMap>`
Loads a custom colormap from the colormaps directory.

#### `load_colormap(name: &str) -> Result<ColorMap>`
Loads a colormap, checking built-ins first, then custom colormaps.

#### `delete_custom_colormap(name: &str) -> Result<()>`
Deletes a custom colormap. Cannot delete built-in colormaps.

#### `list_available_colormaps() -> Result<Vec<ColorMapInfo>>`
Returns information about all available colormaps (built-in + custom).

#### `export_builtin_colormap(name: &str) -> Result<PathBuf>`
Exports a built-in colormap to the custom directory for modification.

### Types

#### `ColorMapError`
Error type for colormap I/O operations:
- `IoError(io::Error)` - File system errors
- `JsonError(serde_json::Error)` - JSON parsing errors
- `NotFound(String)` - ColorMap not found
- `NoConfigDirectory` - Could not determine config directory

#### `ColorMapInfo`
Information about an available colormap:
- `name: String` - Name of the colormap
- `is_builtin: bool` - Whether it's a built-in or custom colormap
- `filepath: Option<PathBuf>` - Path to the file (None for built-ins)

## Implementation Details

### ColorMap Structure Changes

The `ColorMap` struct now includes a `name` field and public `stops` field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorMap {
    pub name: String,
    pub stops: Vec<ColorStop>,
}
```

### Dependencies

- `serde` and `serde_json` - For JSON serialization
- `directories` - For platform-specific config directories

### Testing

Run the tests:
```bash
cargo test --lib colorschemes_io
```

Run the example:
```bash
cargo run --example colormap_io
```

## Best Practices

1. **Always use `load_colormap()`** instead of `load_builtin_colormap()` or `load_custom_colormap()` when you don't know if it's built-in or custom.

2. **Check for errors** - File I/O can fail, so always handle the `Result` properly.

3. **Use meaningful names** - ColorMap names should be descriptive and unique.

4. **Export before modifying built-ins** - If users want to customize a built-in, export it first:
   ```rust
   export_builtin_colormap("Fire")?;
   // Now users can edit it in their config directory
   ```

5. **Validate JSON manually** - If creating JSON files by hand, validate them:
   ```rust
   let json = fs::read_to_string("my_colormap.json")?;
   let colormap: ColorMap = serde_json::from_str(&json)?;
   ```

## Future Enhancements

Possible future additions:
- Import colormap from file path
- Colormap preview/thumbnail generation
- Colormap metadata (author, description, tags)
- Colormap versioning
- Batch import/export
- Colormap presets/categories
