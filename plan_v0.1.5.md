# Plan for v0.1.5 - Import/Export Features & Architecture

## Overview
Version 0.1.5 will focus on round-trip functionality (Load from PNG), code architecture improvements, and potentially new fractal types.

## Priority Order
1. **GUI Architecture Refactoring** (Deferred from v0.1.4) - Do this FIRST before adding new features
2. **Load from PNG Metadata** - Major feature requiring GUI changes
3. **Additional fractals** - Optional if time permits

## Architecture Improvements

### GUI Architecture Refactoring
**Priority**: CRITICAL - Do before other features  
**Status**: Deferred from v0.1.4  
**Rationale**: Current FractalApp struct has 30+ fields scattered across multiple concerns, making maintenance difficult and error-prone

**Current State Analysis:**
The `FractalApp` struct in [main.rs](src/main.rs) (lines 177-243) contains these categories:
1. **View State** (2 fields): `view`, `fractal_texture`
2. **UI Text Inputs** (10 fields): `width_input`, `height_input`, `iterations_input`, `julia_c_real_input`, `julia_c_imag_input`, `mandelbrot_power_input`, `period_input`, `interior_color_r_text`, `interior_color_g_text`, `interior_color_b_text`, `export_scale_input`, `export_supersample_input`
3. **Debouncing** (2 fields): `input_debounce_timer`, `pending_redraw`
4. **Fractal Configuration** (3 fields): `fractal_type`, `fractal_parameters`, `needs_redraw`
5. **Colormap State** (4 fields): `available_colormaps`, `selected_colormap_name`, `colormap`, `color_editor`
6. **Color Modulation** (7 fields): `use_period`, `use_interior_color`, `interior_color`, `use_log_scale` (plus text inputs)
7. **Mouse Interaction** (3 fields): `is_dragging`, `zoom_square_center`, `zoom_square_size`
8. **Export Settings** (3 fields): `export_directory`, `export_filter`, `export_supersample_input`
9. **Status** (1 field): `status_message`

**Problems:**
- Function signatures in `gui.rs` take 10-15 parameters each
- Adding new features requires touching many places
- Parameter passing is error-prone and verbose
- Hard to maintain consistency across functions
- Examples (like mandelpath.rs) duplicate much of this structure

**Proposed Solution - Grouped State Pattern:**

Create logical groupings in `src/app_state.rs`:

```rust
/// View and rendering state
pub struct ViewState {
    pub view: FractalView,
    pub fractal_texture: Option<egui::TextureHandle>,
    pub needs_redraw: bool,
}

/// Text input state with debouncing
pub struct InputState {
    pub width: String,
    pub height: String,
    pub iterations: String,
    pub julia_c_real: String,
    pub julia_c_imag: String,
    pub mandelbrot_power: String,
    pub period: String,
    pub export_scale: String,
    pub export_supersample: String,
    
    // Debouncing
    pub debounce_timer: Option<Instant>,
    pub pending_redraw: bool,
}

/// Fractal configuration
pub struct FractalState {
    pub fractal_type: FractalType,
    pub parameters: HashMap<String, f64>,
}

/// Colormap and modulation settings
pub struct ColorState {
    pub available_colormaps: Vec<String>,
    pub selected_colormap_name: String,
    pub colormap: ColorMap,
    pub color_editor: ColorEditor,
    
    // Modulation
    pub use_period: bool,
    pub use_interior_color: bool,
    pub interior_color: [u8; 3],
    pub interior_color_rgb_text: [String; 3],
    pub use_log_scale: bool,
}

/// Mouse interaction state
pub struct MouseState {
    pub is_dragging: bool,
    pub zoom_square_center: Option<egui::Pos2>,
    pub zoom_square_size: f32,
}

/// Export configuration
pub struct ExportState {
    pub directory: Option<std::path::PathBuf>,
    pub filter: FilterType,
}

/// Main application state
pub struct FractalApp {
    pub view: ViewState,
    pub input: InputState,
    pub fractal: FractalState,
    pub color: ColorState,
    pub mouse: MouseState,
    pub export: ExportState,
    pub status_message: String,
}
```

**Benefits:**
- GUI functions take 3-5 parameters instead of 10-15
- Clear ownership and organization
- Easier to pass state to examples
- Simpler to add new features
- Better encapsulation and maintainability
- Can add helper methods to each state struct

**Implementation Steps:**
- [ ] Create `src/app_state.rs` with state structs
- [ ] Migrate `FractalApp` fields to new structure
- [ ] Update `gui.rs` functions to take grouped state
- [ ] Add helper methods to state structs (e.g., `InputState::parse_iterations()`)
- [ ] Update `main.rs` to use new structure
- [ ] Update examples to reuse state structs
- [ ] Test all functionality still works
- [ ] Document the new pattern in AGENTS.md

**Estimated Impact:**
- Code reduction: ~100-200 lines (parameter lists simplified)
- Function signature improvement: 10-15 params → 3-5 params
- Maintenance improvement: Single source of truth for related state
- Future feature additions: Much easier with grouped state

## New Features

### Load from PNG Metadata
**Priority**: High  
**Status**: Not started  
**Dependencies**: PNG metadata export (completed in v0.1.2)

**Description**: Add "Load from PNG" button to import all settings from exported images with embedded metadata.

**Implementation:**
- [ ] Add "Load from PNG" button in GUI (near export section)
- [ ] Parse PNG tEXt chunks using png crate
- [ ] Deserialize metadata JSON from tEXt chunks
- [ ] Apply all loaded settings:
  - Switch to correct fractal type
  - Load fractal parameters (Julia c values, etc.)
  - Set view coordinates and zoom
  - Restore colormap (including custom color stops)
  - Apply color modulation settings (period, interior color, log scale)
  - Set export settings (filter, supersample, scale)
- [ ] Handle missing/invalid metadata gracefully
- [ ] Show success/error message to user
- [ ] Add file picker dialog for PNG selection

**Benefits:**
- Complete round-trip: Export → Load → Exact reproduction
- Share discoveries with full settings embedded
- Easy iteration on exported images
- No manual settings transcription needed

**Technical Notes:**
- Use rfd::FileDialog for PNG selection
- Validate fractal type and parameters before applying
- Consider adding "Preview Metadata" option to show settings before loading

## Future Considerations
- Support for batch loading multiple PNGs
- Playlist/gallery mode to cycle through saved fractals
- Export/import of just settings (JSON without image)


### Tertation Fractal
**Priority**: Medium  
**Status**: Research phase  
**Reference**: https://www.reddit.com/r/fractals/comments/1q6dh7e/tertation_fractal_z_square_rotation/

**Formula**: z = exp(2π(n+1)i/k) × c × z; k = [1,2]; Default = 1

**Implementation:**
- [ ] Research formula and iteration behavior
- [ ] Create `src/fractals/tertation.rs`
- [ ] Implement `Fractal` trait
- [ ] Define parameters (k value, rotation)
- [ ] Determine good default view coordinates
- [ ] Add to FractalType enum
- [ ] Test rendering and convergence