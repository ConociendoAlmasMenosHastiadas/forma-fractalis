## overview
steps to be caried out are found in the plan*.md files.  If you have not context you should read through the existing context files to get an idea of where the project is going.  This project is heavily created through LLMs.  If during working on this project you spot a rule that might be helpful to include in the agents.md then please indicate this.

## working through a plan
- when working through a plan you should update progress and implementation notes
- if plan results in a feature being deprecated it should start a deprecation plan that will span at least one more feature (so if its marked for deprecation in plan v0.1.3 then plan v0.1.4 should be updated to reflect that removing the feature will be completed)
- **Each release requires a new fractal and colormap**: This is a project requirement for every version. If the plan doesn't specify them at creation, they should be marked as "TBD" and added before finalizing the release. The fractal should showcase the version's features, and the colormap should complement it visually.

## Plan Integration and Dependencies

The plans build on each other in a carefully orchestrated sequence. Always review dependencies before starting a new plan:

**v0.1.5 - GUI Architecture Refactoring** (Foundation)
- Creates grouped state pattern (ViewState, FractalState, ColorState, etc.)
- Reduces function parameters from 10-15 to 3-5
- Prepares architecture for all future features
- **No dependencies** - Can start immediately

**v0.1.6 - PNG Loading** (Builds on v0.1.5)
- Depends on: v0.1.5 state structs
- Loads fractal settings from PNG metadata
- Uses grouped state for cleaner loading logic
- Completes round-trip: Export → Load → Reproduce

**v0.1.7 - Command-Line Rendering** (Builds on v0.1.5 + v0.1.6)
- Depends on: v0.1.5 state structs, v0.1.6 metadata parsing
- Headless rendering from PNG or JSON files
- Reuses state construction patterns
- Enables automation and batch processing

**v0.2.0 - GPU Acceleration** (Major release, builds on all prior)
- Depends on: v0.1.5 state management, v0.1.7 CLI for batch rendering
- GPU compute shaders for parallel iteration
- Auto-switch between CPU/GPU based on workload
- Massive performance improvement for high-res exports

**v0.2.1 - Animated GIF Generation** (Builds on v0.1.7 + v0.2.0)
- Depends on: v0.1.7 batch rendering, v0.2.0 GPU for speed
- Frame generation for zoom/parameter animations
- Leverages GPU for faster frame rendering

**v0.2.2 - Arbitrary Precision** (Research-heavy, builds on v0.2.0)
- Depends on: v0.2.0 GPU architecture understanding
- Arbitrary precision for deep zooms beyond f64 limits
- CPU/GPU switching strategy for extreme zoom

**Key Integration Points:**
1. **State Structs (v0.1.5)** → Used by v0.1.6 loading, v0.1.7 CLI, v0.2.0 GPU
2. **Metadata System (v0.1.6)** → Used by v0.1.7 CLI, enables v0.2.1 keyframes
3. **Batch Rendering (v0.1.7)** → Foundation for v0.2.1 animations
4. **GPU Backend (v0.2.0)** → Accelerates v0.2.1 animations, informs v0.2.2 precision strategy

**When starting a new plan:**
- Read the Dependencies section first
- Verify all prerequisite plans are complete
- Review how the plan integrates with earlier work
- Update cross-references if you discover new integration points

## Grouped State Architecture (v0.1.5+)

Starting with v0.1.5, the application uses a **grouped state pattern** to organize the ~30 individual fields previously scattered across the main FractalApp struct.

### State Structs (defined in src/app_state.rs)

**ViewState** - Fractal view and rendering state
- `view: FractalView` - View coordinates, zoom, dimensions
- `fractal_texture: Option<TextureHandle>` - Rendered fractal texture
- `needs_redraw: bool` - Flag to trigger re-rendering

**InputState** - All text input fields and debouncing
- `width`, `height`, `iterations`, `period`, `bailout`, etc. (String fields)
- `debounce_timer: Option<Instant>` - Input debouncing timer
- `pending_redraw: bool` - Flag for debounced redraw
- Helper methods: `parse_iterations()`, `parse_period()`, `parse_bailout()`, etc.

**FractalState** - Fractal type and parameters
- `fractal_type: FractalType` - Current fractal (Mandelbrot, Julia, etc.)
- `parameters: HashMap<String, f64>` - Fractal-specific parameters
- Methods: `set_parameter()`, `get_parameter()`, fractal lifecycle callbacks

**ColorState** - Color scheme and modulation
- `colormap: ColorMap` - Current color scheme
- `use_period`, `use_interior_color`, `use_log_scale` - Modulation flags
- `interior_color: [u8; 3]` - RGB for interior color
- `interior_color_rgb_text: [String; 3]` - Text inputs for RGB values
- `available_colormaps`, `selected_colormap_name` - Colormap selection

**MouseState** - Mouse interaction for zooming
- `is_dragging: bool` - Currently dragging for zoom
- `zoom_square_center: Option<Pos2>` - Center of zoom square
- `zoom_square_size: f32` - Size of zoom square in pixels

**ExportState** - Export directory and filtering
- `export_directory: String` - Path for saving exports
- `filter_type: FilterType` - PNG filter type selection

### Usage Guidelines

**When adding new state:**
1. Determine which logical group it belongs to
2. Add the field to the appropriate state struct
3. Update Default impl if needed
4. Add helper methods if the field requires parsing or validation

**When refactoring existing code:**
1. Replace individual field references: `self.view` → `self.view_state.view`
2. Use helper methods for parsed inputs: `self.input.parse_iterations()` instead of inline parsing
3. Array splitting for multiple mutable borrows: `let [r, g, b] = &mut self.color.interior_color_rgb_text;`

**Bridge Pattern:**
- main.rs currently uses a bridge pattern: extracts individual fields from grouped state to call old gui.rs function signatures
- This allows gradual refactoring without breaking existing code
- Future work: refactor gui.rs functions to accept grouped state directly

**Benefits:**
- Reduced parameter counts: 10-15 parameters → 3-5
- Clearer code organization and intent
- Easier to pass state to helper functions
- Better IDE autocomplete and navigation
- Foundation for serialization (PNG loading, CLI rendering)

### Example Migration

**Before:**
```rust
struct FractalApp {
    view: FractalView,
    needs_redraw: bool,
    iterations_input: String,
    width_input: String,
    // ... 25+ more fields
}

gui::render_settings(
    ui,
    &mut self.iterations_input,
    &mut self.period_input,
    &mut self.bailout_input,
    // ... 12+ more parameters
);
```

**After:**
```rust
struct FractalApp {
    view_state: ViewState,
    input: InputState,
    fractal: FractalState,
    color: ColorState,
    mouse: MouseState,
    export: ExportState,
    status_message: String,
}

// Eventually (after gui.rs refactoring):
gui::render_settings(ui, &mut self.input, &mut self.fractal);
```

## Adding New Fractals

When adding a new fractal type to the project, follow this checklist to ensure complete integration:

### 1. Create the Fractal Implementation
**File**: `src/fractals/your_fractal.rs`

- [ ] Implement the `Fractal` trait with required methods:
  - `iterate(c_real, c_imag, parameters, max_iter)` - Core iteration logic
  - `default_view(width, height)` - Starting view position/zoom
  - `name()` - Display name for the fractal
  - `parameters()` - Return `Vec<Parameter>` for any configurable parameters
- [ ] Add struct with any internal state (or use zero-sized type if stateless)
- [ ] Add `new()` constructor and `Default` implementation
- [ ] Add unit tests in a `#[cfg(test)]` module

### 2. Register in Module System
**File**: `src/fractals/mod.rs`

- [ ] Add `pub mod your_fractal;` to module declarations
- [ ] Add `pub use your_fractal::YourFractal;` to re-exports

### 3. Add to FractalType Enum
**File**: `src/app_state.rs`

- [ ] Add variant to `FractalType` enum (e.g., `YourFractal`)
- [ ] Update `as_str()` method with display name
- [ ] Update `all()` method to include new variant
- [ ] Update `create_instance()` to instantiate your fractal
- [ ] Update `reset_view_and_params()` with default view and parameters

### 4. Add Input Fields (if parameters exist)
**File**: `src/app_state.rs` - `InputState` struct

- [ ] Add String field(s) for each parameter (e.g., `your_fractal_param: String`)
- [ ] Update `Default` impl with sensible default values
- [ ] Add parser helper method(s) (e.g., `parse_your_fractal_param() -> f64`)

### 5. Update GUI (if parameters exist)
**File**: `src/gui.rs`

- [ ] Add parameter(s) to `render_fractal_settings()` function signature
- [ ] Add parameter(s) to `FractalTypeOps::reset_view_and_params()` trait method
- [ ] Add UI section for your fractal (copy pattern from Mandelbrot or Julia sections):
  ```rust
  if fractal_type.get_name() == "Your Fractal Name" {
      // Add sliders, text inputs, etc.
  }
  ```
- [ ] Update `reset_view_and_params` call to include new parameters

**File**: `src/app_state.rs` - `FractalTypeOps` implementation

- [ ] Update `reset_view_and_params()` implementation to handle new parameters

### 6. Update Main Application
**File**: `src/main.rs`

- [ ] Add import: `use forma_fractalis::fractals::YourFractal;`
- [ ] Add to both fractal match statements (one in `render_fractal()`, one in export section):
  ```rust
  let your_fractal = YourFractal::new();
  // In match:
  FractalType::YourFractal => &your_fractal,
  ```
- [ ] Update `gui::render_fractal_settings()` call with new parameter(s)

### 7. Testing Checklist
- [ ] `cargo check` passes without errors
- [ ] `cargo test` passes all tests including new fractal tests
- [ ] `cargo build --release` succeeds
- [ ] Application runs and new fractal appears in dropdown
- [ ] Parameter controls appear when fractal is selected
- [ ] Parameter changes trigger re-render correctly
- [ ] Fractal renders expected output for test coordinates
- [ ] Export functionality works with new fractal

### 8. Documentation (Optional but Recommended)
- [ ] Add mathematical formula to fractal module docstring
- [ ] Document parameter meanings and ranges
- [ ] Add example usage in comments
- [ ] Update README.md if fractal is release-worthy

**Common Pitfalls:**
- Forgetting to add to both match statements in main.rs (compile will catch)
- Not updating all enum match arms (compile will catch with exhaustive matching)
- Forgetting to expose parameters in GUI (runtime issue - parameter won't be controllable)
- Not calling trigger_debounced_redraw for text inputs (parameter changes won't trigger render)

## Final to-do before each release
- only executed after the plan for the release is complete and you should ask if we're at the end stage
- summarize plan and add to release section in readme.md
- move old plan to old_plans
- use scripts in build_scripts to construct a release
- git commits. Include the command that creates the git releases tagged with versions.

## Benchmarks
The benchmark results can now be referenced easily between versions. When you update to v0.1.4 or later, simply run:
```powershell
cargo bench --bench fractal_bench
```
And copy the relevant results into BENCHMARKS.md for comparison

## Adding New Colormaps

### Using the Coolors Parser Utility

1. **Get palette from coolors.co:**
   - Visit https://coolors.co/ and create or find a palette
   - Click the menu (three dots) → Export → XML
   - Copy the XML content

2. **Save the XML:**
   ```powershell
   # Create a temporary file with the palette XML
   # Example: build_scripts/my_palette.xml
   ```

3. **Generate the colormap JSON:**
   ```powershell
   python build_scripts/coolors_parser.py --pretty \
       -i build_scripts/my_palette.xml \
       -o src/colormaps/my_colormap.json
   ```

4. **Edit the generated JSON:**
   - Open `src/colormaps/my_colormap.json`
   - Change the `"name"` field to a user-friendly name (e.g., "Electric Neon")
   - The parser auto-generates a name from color names, but you should customize it

5. **Register the colormap in the code:**
   - Open `src/colorschemes_io.rs`
   - Find the `define_builtin_colormaps!` macro invocation (around line 60)
   - Add your colormap to the list following the pattern:
     ```rust
     "My Colormap Name" => MY_COLORMAP_JSON => "colormaps/my_colormap.json",
     ```
   - **That's it!** The macro automatically:
     - Creates the constant with `include_str!`
     - Adds it to the `load_builtin_colormap()` function
     - Adds it to the `is_builtin_colormap()` check
     - Adds it to the GUI dropdown list via `list_available_colormaps()`
   - No need to manually update multiple functions anymore!

6. **Test the colormap:**
   ```powershell
   cargo build --release
   .\target\release\forma-fractalis.exe
   ```
   - Select your new colormap from the dropdown
   - Verify colors look correct on a fractal

7. **Clean up:**
   ```powershell
   # Remove the temporary XML file if desired
   rm build_scripts/my_palette.xml
   ```

### Manual Colormap Creation

If you prefer to create colormaps manually without the parser:

1. **Create JSON file:** `src/colormaps/my_colormap.json`

2. **Structure:**
   ```json
   {
     "name": "Display Name",
     "gradient": [
       {"position": 0.0, "r": 255, "g": 0, "b": 0},
       {"position": 0.5, "r": 0, "g": 255, "b": 0},
       {"position": 1.0, "r": 0, "g": 0, "b": 255}
     ]
   }
   ```

3. **Guidelines:**
   - `position`: Must be between 0.0 and 1.0, sorted ascending
   - Include at least 2 gradient stops
   - RGB values: 0-255
   - More stops = smoother transitions

4. **Register the colormap (same as step 5 above):**
   - Add one line to `define_builtin_colormaps!` macro in `src/colorschemes_io.rs`
   - See "Using the Coolors Parser Utility" step 5 for details

### Colormap Best Practices

- **Name conventions:** Use descriptive names (e.g., "Ocean Depths", "Fire Storm")
- **Positions:** Evenly distribute for smooth gradients, or cluster for sharp transitions
- **Colors:** Choose colors with good contrast for visibility
- **Testing:** Test with different fractals (Mandelbrot, Julia) and iteration counts
- **Inspiration:** Paul Bourke's fractal gallery, nature photography, artwork