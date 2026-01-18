# Plan for v0.1.4 - Performance Analysis & Feature Enhancements

## Overview
Version 0.1.4 will focus on identifying and resolving performance bottlenecks introduced or exposed in v0.1.3, along with GUI improvements and code organization to make the codebase more maintainable and easier to extend with examples and experiments.

## Performance Analysis & Optimization

### Performance Concerns from v0.1.3
**Priority**: CRITICAL  
**Status**: Investigation phase  
**Issue**: Potential performance regression introduced in v0.1.3 - need to identify bottlenecks and streamline the rendering pipeline.

### Suspected Changes That May Impact Performance

#### 1. Tippets Mandelbrot Complexity
**Concern**: The Tippets formula includes a `1/z` term which adds:
- Complex division operation per iteration
- Magnitude calculation: `|z| = sqrt(x² + y²)`
- Conditional check for singularity (z near zero)

**Baseline Comparison Needed:**
- [ ] Benchmark standard Mandelbrot (control)
- [ ] Benchmark Tippets Mandelbrot (test)
- [ ] Compare iteration counts to divergence
- [ ] Measure CPU time per pixel

**Expected Impact**: ~10-30% slower than standard Mandelbrot due to division

#### 2. Mouse Interaction Changes
**Concern**: New pointer state checking on every frame:
```rust
let pointer_state = ui.input(|i| i.pointer.clone());
let primary_down = pointer_state.primary_down();
let secondary_down = pointer_state.secondary_down();
```

**Analysis Needed:**
- [ ] Verify this happens only in CentralPanel, not on every UI element
- [ ] Check if `.clone()` on pointer state is expensive
- [ ] Profile frame time with and without interaction checking

**Expected Impact**: Minimal (< 1ms per frame) - pointer state is lightweight

#### 3. Pipeline Redundancy Analysis
**Concern**: Potential redundant work in the rendering pipeline that could be eliminated.

**Areas to Investigate:**

##### Texture Upload Redundancy
- [ ] **Verify**: Are we recreating textures unnecessarily?
  - Check if `load_texture()` is called when texture already exists
  - Verify texture is only updated on `needs_redraw = true`
  - Ensure no texture recreation on zoom square drawing

##### Pixel Buffer Copies
- [ ] **Profile**: Count buffer allocations per render
  - `rendering_pipeline.rs`: Is the RGBA buffer allocated fresh each time?
  - Are we copying pixel data multiple times?
  - Can we use mutable buffers instead of allocating new?

##### Fractal Calculation Overhead
- [ ] **Analyze**: Unnecessary recalculations
  - Does zooming force full image recomputation? (Expected: yes)
  - Are we computing escape iterations for pixels outside view? (Should be: no)
  - Can we cache any calculations between frames? (Probably not worth it)

##### ColorMap Application
- [ ] **Review**: Color mapping efficiency
  - Is gradient interpolation optimized?
  - Are we doing unnecessary conversions (f32 ↔ u8)?
  - Can we pre-compute color tables for common gradients?

##### Filtering/Upscaling Pipeline
- [ ] **Benchmark**: Lanczos and other filters
  - How much time does each filter add?
  - Are filter implementations optimal?
  - Can we optimize filter kernel applications?

### Profiling Strategy

#### Phase 1: Establish Baselines
**Goal**: Create performance baseline measurements for comparison

**Metrics to Collect:**
- [ ] Time to render 1280x720 @ 256 iterations (Mandelbrot)
- [ ] Time to render 1280x720 @ 256 iterations (Tippets)
- [ ] Time to render 1280x720 @ 256 iterations (Julia)
- [ ] Time to render 1280x720 @ 256 iterations (Burning Ship)
- [ ] Frame rate during zoom interaction
- [ ] Texture upload time (GPU transfer)
- [ ] Time spent in `render_with_config()`
- [ ] Time spent in fractal calculation vs color mapping

**Profiling Tools:**
```rust
// Add timing instrumentation
use std::time::Instant;

let start = Instant::now();
// ... operation ...
let elapsed = start.elapsed();
println!("Operation took: {:.2?}", elapsed);
```

**Profiling Points to Add:**
- [ ] Start of `render_with_config()`
- [ ] Start/end of fractal iteration loop
- [ ] Start/end of color mapping
- [ ] Start/end of filtering
- [ ] Start/end of texture upload

#### Phase 2: Identify Hotspots
**Goal**: Find where the most time is spent

**Analysis Tasks:**
- [ ] Run with `--release` profile for realistic measurements
- [ ] Test with different resolutions (640x480, 1280x720, 1920x1080, 3840x2160)
- [ ] Test with different iteration counts (64, 128, 256, 512, 1024)
- [ ] Test with different fractals (Mandelbrot vs Tippets vs Julia)
- [ ] Test with different zoom levels (default, 10x, 100x, 1000x)
- [ ] Compare filter times (None, Box, Lanczos3, Lanczos5)

**Expected Findings:**
- Fractal iteration should be 70-90% of total time
- Color mapping should be 5-10%
- Filtering (if enabled) should be 10-20%
- Texture upload should be < 5%
- UI overhead should be < 5%

**Red Flags to Look For:**
- ⚠️ Texture recreation on every frame (should only be on redraw)
- ⚠️ Multiple buffer allocations per render
- ⚠️ Unnecessary type conversions in hot loops
- ⚠️ Redundant calculations (e.g., computing zoom scale multiple times)
- ⚠️ Synchronous texture uploads blocking UI thread

#### Phase 3: Optimization Implementation
**Goal**: Apply targeted optimizations based on profiling data

**Potential Optimizations:**

##### 1. Parallelize Fractal Calculation (High Impact)
```rust
use rayon::prelude::*;

// Convert pixel iteration to parallel
pixels.par_iter_mut().enumerate().for_each(|(idx, pixel)| {
    let x = idx % width;
    let y = idx / width;
    let (real, imag) = view.screen_to_complex(x, y);
    let iterations = fractal.iterate(real, imag, max_iterations);
    *pixel = map_to_color(iterations, &colormap);
});
```

**Implementation:**
- [ ] Add `rayon` dependency to Cargo.toml
- [ ] Modify `rendering_pipeline.rs` to use parallel iterators
- [ ] Benchmark single-threaded vs parallel (expect 2-4x speedup on quad-core)
- [ ] Ensure thread safety of Fractal trait implementations

##### 2. Optimize Tippets Mandelbrot Calculation
```rust
// Current (possibly slow):
let z_mag = (x * x + y * y).sqrt();
if z_mag > 1e-10 {
    let recip = Complex64::new(x, y) / z_mag.powi(2);
    // ...
}

// Optimized:
let z_mag_sq = x * x + y * y;
if z_mag_sq > 1e-20 {
    let recip_x = x / z_mag_sq;
    let recip_y = -y / z_mag_sq;
    // ... use recip_x, recip_y directly
}
```

**Benefits:**
- Eliminate `sqrt()` call (expensive)
- Eliminate Complex64 division (can do manually)
- Use squared magnitude comparison (cheaper)

##### 3. Color Mapping Optimization
- [ ] Pre-compute periodic color tables when period is enabled
- [ ] Cache interior color as u32 RGBA to avoid repeated packing
- [ ] Use lookup tables instead of gradient interpolation for common cases

##### 4. Reduce Texture Recreation
- [ ] Ensure texture is only recreated when resolution changes
- [ ] Add debug logging to track texture creation events
- [ ] Reuse texture handle between redraws

##### 5. Buffer Pooling
- [ ] Reuse RGBA buffers instead of allocating new ones
- [ ] Keep previous buffer and only resize when dimensions change
- [ ] Consider using `Vec::with_capacity()` with expected max size

### Performance Testing Checklist

**After Each Optimization:**
- [ ] Measure impact with profiling
- [ ] Verify correctness (visual inspection of rendered fractals)
- [ ] Test all fractal types
- [ ] Test at multiple resolutions
- [ ] Test with different iteration counts
- [ ] Ensure no regression in other areas

**Success Criteria:**
- 🎯 1280x720 @ 256 iterations renders in < 100ms (Mandelbrot)
- 🎯 Smooth 60 FPS during zoom interaction
- 🎯 No UI lag during text input (see GUI Performance section)
- 🎯 Tippets Mandelbrot within 50% of standard Mandelbrot speed
- 🎯 No unnecessary texture uploads or buffer allocations

**Regression Prevention:**
- [ ] Add performance benchmarks to test suite
- [ ] Document expected render times in README
- [ ] Add `--release` profile optimization flags if missing

### Code Profiling Setup

**Status**: ✅ IMPLEMENTED (Jan 18, 2026)  
**Implementation**: Added timing instrumentation to identify performance characteristics

**What Was Done:**
- Added `use std::time::Instant` to `rendering_pipeline.rs` and `main.rs`
- Instrumented `render_with_config()` to measure:
  - Buffer allocation time
  - Fractal render time
  - Total pipeline time
- Instrumented `render_fractal()` in main.rs to measure:
  - Image conversion time (RGBA → egui::ColorImage)
  - Texture upload time (GPU transfer)
  - Total GUI overhead
- Performance logging only for preview renders (not export to avoid spam)

**Performance Findings (Jan 18, 2026):**
- ✅ 1280x720 @ 256 iterations: ~9-13ms render, ~12-16ms total (60-80 FPS capable)
- ✅ 1280x720 @ 16384 iterations: ~466-477ms render (expected for high iteration count)
- ✅ Buffer allocation: ~10-23µs (negligible)
- ✅ Image conversion: ~2.5ms (acceptable)
- ✅ Texture upload: ~3-7µs (negligible)
- ✅ No unexpected bottlenecks identified
- ✅ rayon parallelization is working correctly

**Conclusion**: Performance is actually very good. No critical issues found. The perceived slowness may have been from high iteration counts or other factors. Profiling instrumentation can be kept for future debugging or removed if desired.

**TODO for v0.1.4**: Move profiling output behind a `--profiling` command-line flag to avoid console spam during normal use. Consider using `clap` or `std::env::args()` for argument parsing.

**Option 1: Manual Timing (Simple) - IMPLEMENTED**
```rust
// Add to src/lib.rs or rendering_pipeline.rs
pub struct PerfTimer {
    label: &'static str,
    start: Instant,
}

impl PerfTimer {
    pub fn new(label: &'static str) -> Self {
        Self { label, start: Instant::now() }
    }
}

impl Drop for PerfTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        println!("[PERF] {}: {:.2?}", self.label, elapsed);
    }
}

// Usage:
let _timer = PerfTimer::new("Fractal render");
```

**Option 2: Cargo Flamegraph (Advanced)**
```powershell
cargo install flamegraph
cargo flamegraph --root
# Run the app and interact
# Generates flamegraph.svg
```

**Option 3: Built-in Profiler (Windows)**
```powershell
# Use Windows Performance Analyzer
# Or Visual Studio profiler
```

### Implementation Priority

**Phase 1 (Investigation):**
1. Add timing instrumentation to key functions
2. Collect baseline measurements
3. Identify top 3 bottlenecks

**Phase 2 (Quick Wins):**
1. Fix any obvious redundancies (texture recreation, etc.)
2. Optimize Tippets formula (remove sqrt if possible)
3. Implement debounced input (see GUI Performance section)

**Phase 3 (Major Optimization):**
1. Parallelize with rayon (biggest impact)
2. Optimize color mapping
3. Buffer pooling

**Phase 4 (Polish):**
1. Add performance benchmarks
2. Document findings
3. Consider GPU acceleration (future v0.2.x)

## GUI Architecture Refactoring

### Encapsulate Common App State Pattern
**Priority**: High  
**Status**: Not started  
**Goal**: Make it easier to create specialized examples (like `mandelpath.rs`) without duplicating all app state and GUI setup code.

**Problem Analysis:**
Currently, creating a new example like `mandelpath.rs` requires:
1. Duplicating ~90% of `FractalApp` struct fields (view, inputs, colormap state, etc.)
2. Reimplementing initialization logic in `Default` impl
3. Copy-pasting render loop, texture management
4. Maintaining parallel versions of similar functionality

**Observed Duplication Between main.rs and mandelpath.rs:**
- View state: `view: FractalView`
- UI inputs: `width_input`, `height_input`, `iterations_input`
- Colormap state: `available_colormaps`, `selected_colormap_name`, `colormap`, `color_editor`
- Color modulation: `use_period`, `period_input`, `use_interior_color`, interior color RGB, `use_log_scale`
- Rendering: `fractal_texture`, `needs_redraw`
- Zoom interaction: `is_dragging`, `zoom_square_center`, `zoom_square_size`
- Status: `status_message`

**Proposed Solution: Shared App State Struct**

Create a reusable `FractalAppState` struct that encapsulates common functionality:

```rust
// src/app_state.rs (new file)
pub struct FractalAppState {
    // Core rendering
    pub view: FractalView,
    pub fractal_texture: Option<egui::TextureHandle>,
    pub needs_redraw: bool,
    
    // UI inputs
    pub width_input: String,
    pub height_input: String,
    pub iterations_input: String,
    
    // Colormap
    pub available_colormaps: Vec<String>,
    pub selected_colormap_name: String,
    pub colormap: ColorMap,
    pub color_editor: ColorEditor,
    
    // Color modulation
    pub use_period: bool,
    pub period_input: String,
    pub use_interior_color: bool,
    pub interior_color: [u8; 3],
    pub interior_color_r_text: String,
    pub interior_color_g_text: String,
    pub interior_color_b_text: String,
    pub use_log_scale: bool,
    
    // Zoom interaction
    pub is_dragging: bool,
    pub zoom_square_center: Option<egui::Pos2>,
    pub zoom_square_size: f32,
    
    // Status
    pub status_message: String,
}

impl FractalAppState {
    pub fn new(width: u32, height: u32) -> Self { /* ... */ }
    
    pub fn render_sidebar_common(&mut self, ui: &mut egui::Ui) {
        // Dimensions section
        gui::render_dimensions_section(/* ... */);
        
        // Colormap section
        gui::render_colormap_section(/* ... */);
        
        // View info section
        // etc.
    }
    
    pub fn handle_zoom_interaction(
        &mut self,
        response: &egui::Response,
        texture_size: egui::Vec2,
        rect: egui::Rect,
        scale: f32,
        ui: &egui::Ui,
    ) {
        // Consolidate zoom square logic from both apps
    }
    
    pub fn render_fractal<F: Fractal>(
        &mut self,
        ctx: &egui::Context,
        fractal: &F,
        parameters: &HashMap<String, f64>,
    ) {
        // Shared rendering logic
    }
}
```

**Benefits:**
- Examples only define their unique state (e.g., `iteration_paths` for mandelpath)
- Reduces code duplication by 60-70%
- Easier to maintain consistency across examples
- New examples can be scaffolded quickly
- Bug fixes propagate to all examples automatically

**Implementation Plan:**
- [ ] Create `src/app_state.rs` with `FractalAppState` struct
- [ ] Move common fields from `FractalApp` to `FractalAppState`
- [ ] Implement helper methods for common operations:
  - [ ] `new()` - Initialize with sensible defaults
  - [ ] `render_sidebar_common()` - Render standard sidebar sections
  - [ ] `handle_zoom_interaction()` - Zoom square logic
  - [ ] `render_fractal()` - Generic fractal rendering
  - [ ] `parse_iterations()` - Helper to get validated max_iterations
  - [ ] `parse_period()` - Helper to get validated period
- [ ] Refactor `main.rs` to use `FractalAppState`:
  ```rust
  struct FractalApp {
      state: FractalAppState,
      // App-specific fields:
      fractal_type: FractalType,
      fractal_parameters: HashMap<String, f64>,
      julia_c_real_input: String,
      julia_c_imag_input: String,
      // export fields...
  }
  ```
- [ ] Refactor `examples/mandelpath.rs` to use `FractalAppState`:
  ```rust
  struct MandelPathApp {
      state: FractalAppState,
      // Example-specific fields:
      iteration_paths: Vec<IterationPath>,
  }
  ```
- [ ] Update documentation with example template

**Example Usage After Refactoring:**
```rust
// Creating a new specialized example becomes much simpler:
struct MyExperimentApp {
    state: FractalAppState,  // Gets all standard functionality
    my_special_data: Vec<MyThing>,  // Only define what's unique
}

impl eframe::App for MyExperimentApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Render standard sidebar
        egui::SidePanel::left("sidebar").show(ctx, |ui| {
            self.state.render_sidebar_common(ui);
            
            // Add experiment-specific controls here
            ui.separator();
            ui.heading("Experiment Controls");
            // ...
        });
        
        // Render fractal with standard zoom interaction
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.state.fractal_texture {
                let (rect, response) = /* ... */;
                
                // Standard zoom handling
                self.state.handle_zoom_interaction(&response, /* ... */);
                
                // Add custom overlay rendering
                self.render_my_special_overlay(ui);
            }
        });
    }
}
```

**Non-Goals (Keep it Simple):**
- ❌ Don't create deep inheritance hierarchies
- ❌ Don't abstract away every small detail
- ❌ Don't make it harder to understand what's happening
- ✅ DO make common patterns reusable
- ✅ DO reduce boilerplate in examples
- ✅ DO keep the code readable and maintainable

**Testing Strategy:**
- [ ] Verify main app still works after refactoring
- [ ] Verify mandelpath example still works
- [ ] Create a minimal test example to validate the pattern
- [ ] Ensure no performance regression

## New Features


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

### Other Fractal Candidates
**Resources**: https://paulbourke.net/fractals/ (comprehensive fractal reference)

Potential fractals to explore:
- Newton fractals (root-finding visualization)
- Phoenix fractal
- Barnsley fern (IFS)
- Tricorn (Mandelbar)
- Various multibrot sets (z^3, z^4, etc.)

**Process for adding new fractals:**
1. Research formula and mathematical properties
2. Identify interesting parameter ranges
3. Find good default view coordinates
4. Implement Fractal trait
5. Test convergence and performance
6. Add to GUI selector

## GUI Performance Improvements

### Debounced Text Input for Redraw Triggering
**Priority**: High  
**Status**: Not started  
**Issue**: Currently, every keystroke in text input fields (iterations, dimensions, zoom, etc.) triggers an immediate fractal redraw, causing lag and poor user experience during typing.

**Proposed Solutions:**

#### Option 1: Debouncing with Delay Timer (Recommended)
Implement a debounce mechanism that waits for a short delay after the last keystroke before triggering a redraw.

**Implementation:**
- [ ] Add debounce timer state to `FractalApp` struct
  - Store `last_input_time: Option<Instant>`
  - Store `pending_redraw: bool`
- [ ] Create configurable delay constant (recommended: 500-1000ms)
- [ ] Modify text input handlers:
  - On `.changed()`: Update `last_input_time`, set `pending_redraw = true`, do NOT set `needs_redraw = true`
  - In `update()` loop: Check if elapsed time > delay threshold
  - If threshold met and `pending_redraw == true`: trigger actual redraw
- [ ] Use `ctx.request_repaint_after()` to schedule check
- [ ] Consider adding user preference for debounce delay

**Benefits:**
- Smooth typing experience
- Still feels responsive (500ms is barely noticeable)
- Automatically handles rapid changes
- Works for all text inputs uniformly

**Technical Details:**
```rust
use std::time::{Duration, Instant};

struct FractalApp {
    // ... existing fields
    input_debounce_timer: Option<Instant>,
    pending_redraw: bool,
}

const INPUT_DEBOUNCE_DELAY: Duration = Duration::from_millis(500);

// In text input handler:
if ui.text_edit_singleline(&mut self.width_input).changed() {
    self.input_debounce_timer = Some(Instant::now());
    self.pending_redraw = true;
    // Don't set needs_redraw = true here!
}

// In update() method:
if let Some(timer) = self.input_debounce_timer {
    if timer.elapsed() > INPUT_DEBOUNCE_DELAY && self.pending_redraw {
        self.needs_redraw = true;
        self.pending_redraw = false;
        self.input_debounce_timer = None;
    } else if self.pending_redraw {
        // Keep checking
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
```

#### Option 2: Focus-Based Redraw
Only trigger redraws when text input loses focus (user presses Enter or clicks elsewhere).

**Implementation:**
- [ ] Modify all text input to use `.on_hover_text()` with hint about pressing Enter
- [ ] Use `.lost_focus()` instead of `.changed()` to trigger redraw
- [ ] Optionally allow Enter key to explicitly trigger redraw and move focus

**Benefits:**
- No unnecessary redraws while typing
- Very simple implementation
- User has explicit control over when to redraw

**Drawbacks:**
- Less immediate feedback
- Requires user action (Enter/click away) to see changes
- May feel less responsive than debouncing

#### Option 3: Hybrid Approach (Best of Both)
Combine both approaches:
- Use debouncing for minor changes (iterations, dimensions)
- Use focus-loss for coordinate inputs (where precision matters)
- Allow Enter key to force immediate redraw

**Affected Input Fields:**
- Width/Height inputs
- Max iterations input
- Period input
- Zoom/coordinate inputs (direct entry, if implemented)
- Julia set c_real/c_imag inputs
- Interior color RGB values

**Performance Metrics to Consider:**
- Current redraw time for 1280x720 at 256 iterations: ~50-200ms (varies by fractal)
- Target: No UI lag during typing
- Acceptable delay: 500-1000ms feels natural

**Additional Optimizations:**
- [ ] Consider caching the last valid parsed value to avoid parsing errors during incomplete input
- [ ] Add visual indicator when redraw is pending (e.g., subtle highlight or icon)
- [ ] Make debounce delay configurable in settings (future enhancement)

**Testing Checklist:**
- [ ] Type rapidly in dimension fields - should not lag
- [ ] Type slowly - should see updates within delay period
- [ ] Press Enter in any field - should trigger immediate redraw
- [ ] Click between fields - focus loss should trigger redraw
- [ ] Invalid input (non-numeric) - should not crash or trigger redraw

## Documentation Cleanup

### Reduce README.md
**Priority**: Low  
**Status**: Not started  
**Issue**: README.md has grown quite large with detailed release notes and extensive feature lists

**Proposed Changes:**
- [ ] Consider moving detailed release notes to CHANGELOG.md
- [ ] Consolidate feature descriptions (currently quite verbose)
- [ ] Move technical details to separate documentation files
- [ ] Keep README focused on:
  - Quick overview and screenshots
  - Installation/getting started
  - Basic usage
  - Links to detailed docs
- [ ] Create CHANGELOG.md for version history
- [ ] Consider moving examples documentation to examples/README.md

**Benefits:**
- Faster to read for new users
- Easier to maintain
- More professional presentation
- Separation of concerns (README vs CHANGELOG)

## Resources
- Paul Bourke's Fractal Gallery: https://paulbourke.net/fractals/
