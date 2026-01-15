# Plan for v0.1.3 - Cleanup and Deprecation

## Overview
Version 0.1.3 will focus on removing backward compatibility layers introduced during the v0.1.2 refactoring. This will result in a cleaner, more maintainable codebase.

## Deprecation Schedule

### High Priority - Remove Backward Compatibility Layers

#### 1. Remove `src/fractal.rs` compatibility layer
**Status**: DEPRECATED in v0.1.2  
**Target**: Remove in v0.1.3  
**Impact**: Medium - still used throughout codebase  

**Current State:**
- Contains `MandelbrotView` struct (old view type)
- Contains `mandelbrot_iterations()` function (delegates to trait)
- Marked as deprecated with doc comments
- Used by: main.rs, gui.rs, export.rs, rendering.rs

**Migration Path:**
- [ ] Replace all `MandelbrotView` with `FractalView` from `fractals` module
- [ ] Update all imports from `fractal::MandelbrotView` to `fractals::FractalView`
- [ ] Remove `mandelbrot_iterations()` calls (already replaced in new rendering path)
- [ ] Delete `src/fractal.rs` file
- [ ] Update `lib.rs` to remove re-export

**Files to Update:**
- `src/main.rs` - Change view type and imports
- `src/gui.rs` - Update function signatures
- `src/export.rs` - Update function signatures
- `src/rendering.rs` - Remove `render_mandelbrot()` function
- `src/lib.rs` - Remove `pub mod fractal;` and re-exports

#### 2. Remove `render_mandelbrot()` from rendering.rs
**Status**: DEPRECATED in v0.1.2  
**Target**: Remove in v0.1.3  
**Impact**: Low - not actively used after Phase 5

**Current State:**
- Old rendering function using `MandelbrotView` and `mandelbrot_iterations()`
- Kept for backward compatibility
- New code uses `render_fractal()` with trait objects

**Migration Path:**
- [ ] Verify no calls to `render_mandelbrot()` exist in codebase
- [ ] Remove function definition from `src/rendering.rs`
- [ ] Remove from module exports if publicly exported

### Medium Priority - Refactoring Improvements

#### 3. Migrate from `MandelbrotView` to `FractalView` everywhere
**Status**: Partially complete  
**Target**: Complete in v0.1.3  
**Impact**: High - touches most of codebase

**Current State:**
- `FractalView` exists in `fractals::mod.rs` with proper parameter support
- `MandelbrotView` still used in app state and function signatures
- Both are functionally similar but `FractalView` has `parameters: HashMap<String, f64>`

**Benefits of Migration:**
- Consistent naming (no "Mandelbrot" in generic code)
- Built-in parameter support for all fractals
- Cleaner API boundary

**Migration Checklist:**
- [ ] `src/main.rs`: Change `view: MandelbrotView` to `view: FractalView` in FractalApp
- [ ] `src/gui.rs`: Update all function signatures accepting `&MandelbrotView`
- [ ] `src/export.rs`: Update `export_png()` signature and implementation
- [ ] `src/rendering_pipeline.rs`: Update `RenderConfig` to use `FractalView`
- [ ] Update conversion logic where `FractalView` needs default view parameters

#### 4. Consolidate fractal instance creation
**Status**: Working but duplicated  
**Target**: Improve in v0.1.3  
**Impact**: Low - code quality improvement

**Current State:**
- Fractal instances created multiple times (render_fractal, export, etc.)
- Pattern: `let mandelbrot = Mandelbrot::new(); let julia = Julia::new(); ...`
- Repeated in `render_fractal()` and `render_actions_section()` calls

**Improvement Options:**
- [ ] Option A: Add helper function to create fractal from FractalType enum
- [ ] Option B: Store current fractal as `Box<dyn Fractal>` in app state
- [ ] Option C: Keep current approach (simple, low memory, works fine)

**Recommended**: Option A - Add helper function
```rust
impl FractalType {
    pub fn create_instance(&self) -> Box<dyn Fractal> {
        match self {
            FractalType::Mandelbrot => Box::new(Mandelbrot::new()),
            FractalType::Julia => Box::new(Julia::new()),
            FractalType::BurningShip => Box::new(BurningShip::new()),
        }
    }
}
```

### Low Priority - Nice to Have

#### 5. Add deprecation warnings
**Status**: Not started  
**Target**: Add in v0.1.2.1 (patch), enforce in v0.1.3  

- [ ] Add `#[deprecated]` attribute to `MandelbrotView`
- [ ] Add `#[deprecated]` attribute to `mandelbrot_iterations()`
- [ ] Add `#[deprecated]` attribute to `render_mandelbrot()`
- [ ] Include migration instructions in deprecation messages

Example:
```rust
#[deprecated(
    since = "0.1.2",
    note = "Use `FractalView` from `crate::fractals` module instead"
)]
pub struct MandelbrotView { ... }
```

#### 6. Documentation updates
**Status**: Not started  
**Target**: v0.1.3  

- [ ] Update README.md to reflect multi-fractal capabilities
- [ ] Add examples of using different fractals
- [ ] Document the trait-based architecture
- [ ] Update module-level docs to reflect new structure

## Testing Requirements for v0.1.3

After removing backward compatibility layers:
- [ ] All fractals render correctly (Mandelbrot, Julia, Burning Ship)
- [ ] Zoom and pan work for all fractals
- [ ] Export works with all fractals
- [ ] Parameter changes work (Julia c values)
- [ ] Fractal switching works seamlessly
- [ ] Color schemes apply correctly
- [ ] Period modulation works
- [ ] Interior color works
- [ ] Log scale works
- [ ] No compilation warnings
- [ ] No panics or errors during normal operation

## Version Timeline

**v0.1.2** (Current - January 2026)
- ✅ Trait-based fractal system implemented
- ✅ Three fractals: Mandelbrot, Julia, Burning Ship
- ✅ GUI selector for fractal types
- ⚠️ Backward compatibility layers present

**v0.1.2.1** (Patch - Optional)
- Add `#[deprecated]` attributes
- Add compiler warnings for old code paths
- No functional changes

**v0.1.3** (Next Release - Target: Q1 2026)
- 🎯 Remove all backward compatibility layers
- 🎯 Complete migration to FractalView
- 🎯 Clean codebase with no deprecated code
- 🎯 Updated documentation

## Success Criteria

Version 0.1.3 is complete when:
1. ✅ `src/fractal.rs` file is deleted
2. ✅ All references to `MandelbrotView` are replaced with `FractalView`
3. ✅ `render_mandelbrot()` function is removed
4. ✅ All tests pass
5. ✅ Application runs without warnings
6. ✅ Export filenames correctly use fractal names
7. ✅ Code is cleaner and more maintainable

## Risk Assessment

**Low Risk Items:**
- Removing `render_mandelbrot()` - not actively used
- Adding deprecation warnings - no breaking changes

**Medium Risk Items:**
- Replacing `MandelbrotView` with `FractalView` - touches many files but straightforward

**Mitigation:**
- Make changes incrementally
- Test after each file update
- Keep git history clean with atomic commits
- Consider feature branch for large refactoring


## New Features for v0.1.3

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

### Additional Fractals

#### Tertation Fractal
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

#### Other Fractal Candidates
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

## NEW Fractals Reference
save https://paulbourke.net/fractals/ you're going to need it
