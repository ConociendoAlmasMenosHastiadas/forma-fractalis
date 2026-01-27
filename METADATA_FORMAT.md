# PNG Metadata Format

## Overview

Forma Fractalis embeds fractal settings as PNG tEXt chunks, enabling complete reproducibility of any exported fractal. This document describes the metadata format used in version 0.1.6+.

## Metadata Version

Current metadata version: **1.0**

The metadata version is separate from the application version and changes only when the metadata format itself changes in a breaking way.

## Metadata Fields

All metadata is stored as key-value pairs in PNG tEXt chunks. Here's a complete reference:

### Version Information

| Key | Type | Description | Example |
|-----|------|-------------|---------|
| `Forma-Fractalis-Version` | String | Application version that created the image | `"0.1.6"` |
| `Metadata-Version` | String | Metadata format version | `"1.0"` |
| `Created-Timestamp` | Integer | Unix timestamp when fractal was created | `"1706227200"` |

### Fractal Configuration

| Key | Type | Description | Example |
|-----|------|-------------|---------|
| `Fractal-Type` | String | Name of the fractal algorithm | `"Mandelbrot"`, `"Julia"`, `"Burning Ship"`, `"Tippets Mandelbrot"`, `"Multifractal-Julia"`, `"Cactus"` |
| `Fractal-Parameters` | JSON | Fractal-specific parameters as a JSON object | `{"c_real": -0.7, "c_imag": 0.27}` |
| `Max-Iterations` | Integer | Maximum iteration count | `"256"` |

### View Parameters

| Key | Type | Description | Example |
|-----|------|-------------|---------|
| `View-CenterX` | Float | X-coordinate of the view center | `"-0.5"` |
| `View-CenterY` | Float | Y-coordinate of the view center | `"0.0"` |
| `View-Zoom` | Float | Zoom level (1.0 = default) | `"1.0"` |
| `View-Width` | Integer | Original view width in pixels | `"1280"` |
| `View-Height` | Integer | Original view height in pixels | `"720"` |

**Note**: `View-Width` and `View-Height` represent the original fractal view dimensions, not the exported PNG dimensions (which may be scaled by `Export-Scale`).

### Color Scheme

| Key | Type | Description | Example |
|-----|------|-------------|---------|
| `Colormap-Name` | String | Name of the colormap | `"Ocean"` |
| `Colormap-Data` | JSON | Full colormap definition with gradient stops | See below |

**Colormap-Data JSON Structure:**
```json
{
  "name": "Ocean",
  "gradient": [
    {"position": 0.0, "r": 0, "g": 0, "b": 128},
    {"position": 0.5, "r": 0, "g": 128, "b": 255},
    {"position": 1.0, "r": 0, "g": 255, "b": 255}
  ]
}
```

### Color Modulation

| Key | Type | Description | Example |
|-----|------|-------------|---------|
| `Color-Period-Enabled` | Boolean | Whether period modulation is enabled | `"true"` |
| `Color-Period` | Integer | Period value for color cycling | `"128"` |
| `Interior-Color-Enabled` | Boolean | Whether custom interior color is used | `"false"` |
| `Interior-Color-RGB` | JSON Array | RGB values for interior color | `[0, 0, 0]` |
| `Log-Scale-Enabled` | Boolean | Whether logarithmic color scaling is enabled | `"false"` |

### Export Settings

| Key | Type | Description | Example |
|-----|------|-------------|---------|
| `Export-Filter` | String | Filter type used for export | `"Lanczos3"`, `"Gaussian"`, `"None"` |
| `Export-Supersample` | Integer | Supersampling multiplier | `"4"` |
| `Export-Scale` | Float | Output scale multiplier | `"3.0"` |

## Fractal-Specific Parameters

### Julia Set
```json
{
  "c_real": -0.7,
  "c_imag": 0.27
}
```

### Mandelbrot Variants
```json
{
  "power": 2.0
}
```

### Multifractal-Julia
```json
{
  "power": 1.0
}
```

### Burning Ship
No additional parameters (empty object `{}`).

### Tippets Mandelbrot
No additional parameters (empty object `{}`).

### Cactus
No additional parameters (empty object `{}`).

Iteration formula: z_{n+1} = z_n^3 + (z_0 - 1)z_n - z_0

## Example Metadata

Here's a complete example of metadata for a Julia set export:

```
Forma-Fractalis-Version: 0.1.6
Metadata-Version: 1.0
Created-Timestamp: 1706227200
Fractal-Type: Julia
Fractal-Parameters: {"c_real":-0.7,"c_imag":0.27}
Max-Iterations: 256
View-CenterX: 0.0
View-CenterY: 0.0
View-Zoom: 1.0
Colormap-Name: Ocean
Colormap-Data: {"name":"Ocean","gradient":[...]}
Color-Period-Enabled: true
Color-Period: 128
Interior-Color-Enabled: false
Interior-Color-RGB: [0,0,0]
Log-Scale-Enabled: false
Export-Filter: Lanczos3
Export-Supersample: 4
Export-Scale: 3.0
```

## Loading Metadata

When loading metadata from a PNG file:

1. **Validation**: The loader checks for `Metadata-Version` or `Forma-Fractalis-Version` to confirm it's a valid Forma Fractalis export
2. **Required Fields**: `Fractal-Type`, view parameters, and colormap data are required
3. **Optional Fields**: If optional fields are missing, sensible defaults are used
4. **Error Handling**: Invalid or corrupted metadata results in a user-friendly error message

## Future Compatibility

### Adding New Fields

New optional fields can be added without breaking compatibility. The loader will ignore unknown fields.

### Breaking Changes

If the metadata format needs breaking changes:
1. Increment `Metadata-Version` (e.g., to `"2.0"`)
2. Implement version-specific loading logic
3. Maintain backwards compatibility for older versions

## Command-Line Usage (v0.1.7+)

Future versions will support command-line rendering from PNG metadata:

```powershell
forma-fractalis --load fractal.png --render
forma-fractalis --load fractal.png --output output.png --scale 5.0
```

This will enable batch processing and automation workflows.

## Technical Notes

- **PNG Chunk Type**: All metadata uses the `tEXt` chunk type (uncompressed text)
- **Encoding**: All text is UTF-8 encoded
- **JSON Format**: JSON values use compact formatting (no unnecessary whitespace)
- **Precision**: Float values are stored with full precision to ensure exact reproduction

## See Also

- [export.rs](src/export.rs) - Export and metadata creation implementation
- [COLORMAP_SAVELOAD.md](COLORMAP_SAVELOAD.md) - Colormap file format documentation
