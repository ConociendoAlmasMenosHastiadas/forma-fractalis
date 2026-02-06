## overview
steps to be caried out are found in the plan*.md files.  If you have not context you should read through the existing context files to get an idea of where the project is going.  This project is heavily created through LLMs.  If during working on this project you spot a rule that might be helpful to include in the agents.md then please indicate this.

## working through a plan
- when working through a plan you should update progress and implementation notes
- if plan results in a feature being deprecated it should start a deprecation plan that will span at least one more feature (so if its marked for deprecation in plan v0.1.3 then plan v0.1.4 should be updated to reflect that removing the feature will be completed)
- **Each release requires a new fractal and colormap**: This is a project requirement for every version. If the plan doesn't specify them at creation, they should be marked as "TBD" and added before finalizing the release. The fractal should showcase the version's features, and the colormap should complement it visually.

## Architecture Notes

**Grouped State Pattern** (implemented in v0.1.5):
The application uses grouped state structs defined in `src/app_state.rs`:
- **ViewState** - Fractal view and rendering
- **InputState** - Text inputs and debouncing (has `parse_*()` helper methods)
- **FractalState** - Fractal type and parameters
- **ColorState** - Colormaps and color modulation
- **MouseState** - Mouse interaction for zooming
- **ExportState** - Export settings

When adding new state, determine which group it belongs to and add helper methods if needed.

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
- only executed after the plan for the release is complete and you should ask if we're at the end stage.  complete means the points are settled or moved into the next plan.
- summarize plan and add to release section in readme.md
- move old plan to plans/old_plans
- use scripts in build_scripts to construct a release
- git commits. Include the command that creates the git releases tagged with versions.

## Benchmarks
The benchmark results can now be referenced easily between versions. When you update to v0.1.4 or later, simply run:
```powershell
cargo bench --bench fractal_bench
```
And copy the relevant results into BENCHMARKS.md for comparison

## Adding New Colormaps

**Since v0.1.61, colormaps are maintained in the [scala-chromatica](https://github.com/ConociendoAlmasMenosHastiadas/scala-chromatica) crate.**

See the scala-chromatica AGENTS.md for complete colormap creation guidelines:
- Coolors.co parser utility
- Manual colormap creation
- HSV gradient techniques
- Best practices and testing

New colormaps added to scala-chromatica automatically appear in forma-fractalis after running `cargo update`.