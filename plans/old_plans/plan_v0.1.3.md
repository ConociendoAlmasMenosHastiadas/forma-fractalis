# Plan for v0.1.3 - Cleanup and Deprecation

## Status: ✅ CORE OBJECTIVES COMPLETED + NEW FEATURE (January 18, 2026)

**Major Accomplishments:**
- ✅ Removed all backward compatibility layers from v0.1.2
- ✅ Successfully migrated MandelbrotView → FractalView throughout codebase
- ✅ Deleted deprecated fractal.rs module
- ✅ Fixed export filename bug and Julia set default view
- ✅ Application compiles and runs without errors
- ✅ All core refactoring objectives achieved
- ✅ **NEW: Added Tippets Mandelbrot fractal as release feature**
- ✅ **NEW: Added Twilight Garden colormap (replaced mint_lavender)**
- ✅ **NEW: Created examples/mandelpath.rs for iteration path visualization**
- ✅ Standardized mouse interactions (left-click only for zoom)
- ✅ Added performance profiling instrumentation

**Remaining Tasks:**
- ✅ All core tasks completed
- ⚠️ Optional: GitHub repo rename (user must do manually)
- ⚠️ Optional: Remove profiling instrumentation (or keep for future debugging)

## Overview
Version 0.1.3 focuses on removing backward compatibility layers introduced during the v0.1.2 refactoring. This results in a cleaner, more maintainable codebase with consistent naming and architecture.

**New Feature:** Tippets Mandelbrot - a variation of the classic Mandelbrot set that adds a reciprocal term to the iteration formula: z(n+1) = z² + c + (1/z)


## bugs identified in v0.1.2
[✓] export of images does not always produce the correct name. Example "julia_set_3840_4xLanczos31768376225.png"
    **FIXED**: Added target_height to filename format string. Now produces: `fractal_name_WIDTHxHEIGHT_filter_timestamp.png`
[✓] julia set default view doesn't seem correctly centered
    **FIXED**: Changed default zoom from 1.0 to 0.7 to better display the full Julia set within the viewport


## Deprecation Schedule
[✓] deprecate "mandelrust" as the name.  move to "forma-fractalis"
 - ✅ edit cargo
 - ✅ edit metadata export
 - ⚠️ GitHub repo rename: User must do manually in GitHub settings

### High Priority - Remove Backward Compatibility Layers

#### 1. Remove `src/fractal.rs` compatibility layer
**Status**: ✅ COMPLETED (Jan 17, 2026)
**Target**: Remove in v0.1.3  
**Impact**: Medium - still used throughout codebase  

**Completed Actions:**
- ✅ Replaced all `MandelbrotView` with `FractalView` from `fractals` module
- ✅ Updated all imports from `fractal::MandelbrotView` to `fractals::FractalView`
- ✅ Removed `mandelbrot_iterations()` calls (removed with render_mandelbrot)
- ✅ Deleted `src/fractal.rs` file
- ✅ Updated `lib.rs` to remove module and re-exports

**Files Updated:**
- ✅ `src/main.rs` - Changed view type and imports
- ✅ `src/gui.rs` - Updated function signatures
- ✅ `src/export.rs` - Updated function signatures and tests
- ✅ `src/rendering.rs` - Removed `render_mandelbrot()` function and updated imports
- ✅ `src/rendering_pipeline.rs` - Updated RenderConfig and tests
- ✅ `src/lib.rs` - Removed `pub mod fractal;` and updated docs
- ✅ `src/fractals/mod.rs` - Added `reset()` method to FractalView for compatibility

**Notes:**
- Added `reset()` method to FractalView to maintain API compatibility with GUI
- All compilation and runtime tests passed successfully

#### 2. Remove `render_mandelbrot()` from rendering.rs
**Status**: ✅ COMPLETED (Jan 17, 2026)
**Target**: Remove in v0.1.3  
**Impact**: Low - not actively used after Phase 5

**Completed Actions:**
- ✅ Verified no calls to `render_mandelbrot()` exist in codebase
- ✅ Removed function definition from `src/rendering.rs`
- ✅ Removed `mandelbrot_iterations()` import and usage
- ✅ Updated imports to use `fractals::FractalView` instead of `fractal::MandelbrotView`

### Medium Priority - Refactoring Improvements

#### 3. Migrate from `MandelbrotView` to `FractalView` everywhere
**Status**: ✅ COMPLETED (Jan 17, 2026)
**Target**: Complete in v0.1.3  
**Impact**: High - touches most of codebase

**Benefits Achieved:**
- ✅ Consistent naming (no "Mandelbrot" in generic code)
- ✅ Built-in parameter support for all fractals
- ✅ Cleaner API boundary

**Migration Checklist:**
- ✅ `src/main.rs`: Changed `view: MandelbrotView` to `view: FractalView` in FractalApp
- ✅ `src/gui.rs`: Updated all function signatures accepting `&MandelbrotView`
- ✅ `src/export.rs`: Updated `export_png()` signature and implementation
- ✅ `src/rendering_pipeline.rs`: Updated `RenderConfig` to use `FractalView`
- ✅ Updated tests to use FractalView::new()

#### 4. Consolidate fractal instance creation
**Status**: ✅ COMPLETED (Jan 17, 2026)
**Target**: Improve in v0.1.3  
**Impact**: Low - code quality improvement

**Implementation:**
- ✅ Added `create_instance()` method to FractalType enum
- ✅ Returns `Box<dyn Fractal>` for the appropriate fractal type
- ✅ Eliminates code duplication when creating fractal instances

**Code Added:**
```rust
impl FractalType {
    pub fn create_instance(&self) -> Box<dyn forma_fractalis::fractals::Fractal> {
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

#### 5. Add deprecation warnings
**Status**: N/A - No longer needed (deprecated code removed)
**Original Target**: v0.1.2.1 (patch)

**Note:** Since we proceeded directly to removal in v0.1.3, deprecation warnings were not needed.

#### 6. Documentation updates
**Status**: ✅ COMPLETED (Jan 17, 2026)
**Target**: v0.1.3  

- ✅ Updated lib.rs module-level docs to reflect multi-fractal architecture
- ✅ Updated README.md with v0.1.3 release notes
- ✅ Updated README.md module structure (removed deprecated fractal.rs)
- ✅ Updated version badge to 0.1.3
- ✅ Documented bug fixes and code cleanup in release notes

## Testing Requirements for v0.1.3

After removing backward compatibility layers:
- ✅ All fractals render correctly (Mandelbrot, Julia, Burning Ship)
- ✅ Zoom and pan work for all fractals
- ✅ Export works with all fractals
- ✅ Parameter changes work (Julia c values)
- ✅ Fractal switching works seamlessly
- ✅ Color schemes apply correctly
- ✅ Period modulation works
- ✅ Interior color works
- ✅ Log scale works
- ✅ No compilation warnings
- ✅ No panics or errors during normal operation

**Testing Notes:**
- Application compiled successfully on first attempt after adding `reset()` method
- Manual runtime testing confirmed all fractals render and interact correctly

## Version Timeline

**v0.1.2** (Released - January 2026)
- ✅ Trait-based fractal system implemented
- ✅ Three fractals: Mandelbrot, Julia, Burning Ship
- ✅ GUI selector for fractal types
- ⚠️ Backward compatibility layers present

**v0.1.2.1** (Patch - Skipped)
- Originally planned for deprecation warnings
- Skipped in favor of direct removal in v0.1.3

**v0.1.3** (In Progress - January 17, 2026)
- ✅ Remove all backward compatibility layers
- ✅ Complete migration to FractalView
- ✅ Clean codebase with no deprecated code
- ✅ Bug fixes (export filenames, Julia default view)
- 🔄 Documentation updates (partial)

## Success Criteria

Version 0.1.3 is complete when:
1. ✅ `src/fractal.rs` file is deleted
2. ✅ All references to `MandelbrotView` are replaced with `FractalView`
3. ✅ `render_mandelbrot()` function is rem
8. ✅ New fractal type (Tippets Mandelbrot) added
9. ✅ New colormap (Twilight Garden) added
10. ✅ Mouse interactions standardized
11. ✅ Performance profiling instrumentation added

**ALL SUCCESS CRITERIA MET** ✅

## Final Checklist for v0.1.3 Release

### Core Functionality
- ✅ Application compiles without errors
- ✅ Application runs without warnings
- ✅ All fractals render correctly (Mandelbrot, Julia, Burning Ship, Tippets)
- ✅ Zoom and pan work for all fractals
- ✅ Export works with all fractals
- ✅ Parameter changes work (Julia c values)
- ✅ Fractal switching works seamlessly
- ✅ Color schemes apply correctly
- ✅ Period modulation works
- ✅ Interior color works
- ✅ Log scale works
- ✅ Mouse interactions work correctly (left-click = zoom, right-click = reserved)

### New Features
- ✅ Tippets Mandelbrot fractal implemented and working
- ✅ Twilight Garden colormap implemented and working
- ✅ examples/mandelpath.rs created and functional
- ✅ Performance profiling instrumentation added

### Code Quality
- ✅ No deprecated code remains
- ✅ Consistent naming throughout (FractalView)
- ✅ Clean module structure
- ✅ Documentation updated

### Performance
- ✅ Rendering performance verified (9-13ms for 1280x720 @ 256 iterations)
- ✅ rayon parallelization confirmed working
- ✅ No critical bottlenecks identified

### Documentation
- ✅ README.md updated with v0.1.3 release notes
- ✅ Cargo.toml version updated to 0.1.3
- ✅ plan_v0.1.3.md completed
- ✅ plan_v0.1.4.md created
- ✅ plan_v0.1.5.md created

### Optional Tasks
- ⚠️ GitHub repository rename (user must do manually in GitHub settings)
- ⚠️ Remove profiling instrumentation (or keep for debugging)

## v0.1.3 Release Ready ✅

All core objectives and success criteria have been met. The release is ready for deployment.oved
4. ✅ All tests pass
5. ✅ Application runs without warnings
6. ✅ Export filenames correctly use fractal names
7. ✅ Code is cleaner and more maintainable

## Risk Assessment

**Completed with Low Risk:**
- ✅ Removing `render_mandelbrot()` - not actively used, removed cleanly
- ✅ Deprecation warnings - skipped entirely, went straight to removal

**Completed Medium Risk Items:**
- ✅ Replacing `MandelbrotView` with `FractalView` - touched many files but was straightforward
  - Only issue: needed to add `reset()` method to FractalView for GUI compatibility
  - All changes compiled and tested successfully

**Mitigation Applied:**
- ✅ Made changes incrementally across multiple files
- ✅ Tested after changes (compilation and runtime)
- ✅ Maintained clean separation between different tasks

## Implementation Summary (January 17, 2026)

### Changes Made:
1. **Bug Fixes:**
   - Fixed export filename format to include height dimension
   - Adjusted Julia set default zoom from 1.0 to 0.7

2. **Core Migration:**
   - Replaced all MandelbrotView → FractalView across 7 files
   - Updated imports in: main.rs, gui.rs, export.rs, rendering.rs, rendering_pipeline.rs
   - Removed deprecated fractal.rs module entirely
   - Updated lib.rs module declarations and documentation

3. **Code Quality:**
   - Added `create_instance()` helper method to FractalType enum
   - Added `reset()` method to FractalView for GUI compatibility
   - Removed render_mandelbrot() function completely

4. **Planning:**
   - Created plan_v0.1.4.md for future features
   - Moved PNG import feature to v0.1.4
   - Moved additional fractals to v0.1.4

### Remaining Tasks:
- ✅ Project renamed to "forma-fractalis"
  - ✅ Updated: Cargo.toml name, README title, window title, metadata exports
  - ✅ Updated: Build scripts, config directories, all code references
  - ✅ Compilation successful, executable created as `forma-fractalis.exe`
  - ⚠️ **USER ACTION REQUIRED**: Rename GitHub repository manually in GitHub settings
    - Go to: https://github.com/ConociendoAlmasMenosHastiadas/mandelrust/settings
    - Change repository name from "mandelrust" to "forma-fractalis"
  - 📋 **OPTIONAL**: Rename local directory from `mandelrust` to `forma-fractalis`

## New Features Added

### Tippets Mandelbrot Fractal
**Status**: ✅ COMPLETED (Jan 18, 2026)

Added a new fractal type to the application - the Tippets Mandelbrot, which is a variation of the classic Mandelbrot set.

**Formula**: z(n+1) = z² + c + (1/z)

This fractal adds a reciprocal term to the standard Mandelbrot iteration, creating interesting variations and distortions in the fractal structure. The implementation includes proper singularity handling for cases where z approaches zero.

**Implementation Details:**
- ✅ Created `src/fractals/tippets_mandelbrot.rs` with full Fractal trait implementation
- ✅ Added module export in `src/fractals/mod.rs`
- ✅ Integrated into `FractalType` enum in `src/main.rs`
- ✅ Added to all match statements for fractal selection
- ✅ Default view: center (-0.5, 0.0), zoom 0.8
- ✅ Singularity protection: skips 1/z term when |z| < 1e-10

**Files Modified:**
- src/fractals/tippets_mandelbrot.rs (new)
- src/fractals/mod.rs
- src/main.rs

### Twilight Garden Colormap
**Status**: ✅ COMPLETED (Jan 18, 2026)

Added a new colormap called "Twilight Garden" featuring a soft, natural palette that transitions from warm peachy tones through mint greens to deep teals and mauve.

**Color Palette:**
- Powder Petal (#FFE2D1) - Soft peachy cream
- Frosted Mint (#E1F0C4) - Light mint green
- Muted Teal (#6BAB90) - Medium teal
- Jungle Teal (#55917F) - Deep teal
- Mauve Shadow (#5E4C5A) - Dark mauve

**Files:**
- ✅ Created: `src/colormaps/twilight_garden.json`
- ✅ Removed: `src/colormaps/mint_lavender.json` (didn't work well with fractals)

### Files Modified (16 total):
- src/main.rs
- src/gui.rs
- src/export.rs
- src/rendering.rs
- src/rendering_pipeline.rs
- src/lib.rs
- src/fractals/mod.rs
- src/fractals/julia.rs
- src/fractals/tippets_mandelbrot.rs (new)
- src/colorschemes_io.rs
- examples/mandelpath.rs (new)
- plan_v0.1.3.md (this file)
- plan_v0.1.4.md
- plan_v0.1.5.md
- README.md
- Cargo.toml

### Files Created (5):
- plan_v0.1.4.md
- plan_v0.1.5.md
- src/fractals/tippets_mandelbrot.rs
- examples/mandelpath.rs
- src/colormaps/twilight_garden.json

### Files Deleted (2):
- src/fractal.rs
- src/colormaps/mint_lavender.json
- Consider feature branch for large refactoring


## Future Improvements

(No major improvements planned for v0.1.3 - see plan_v0.1.4.md for next version features)

**Mouse Interaction Standardization:**
- ✅ COMPLETED (Jan 18, 2026): Standardized mouse interactions across main.rs and examples
- ✅ Left-click drag: Zoom rectangle (both main.rs and examples)
- ✅ Right-click: Reserved for future use in main.rs, used for path generation in examples/mandelpath.rs
- ✅ Scroll wheel: Adjusts zoom rectangle size during drag
- Implementation: Added pointer state checks to ensure only primary button triggers zoom rectangle

**Performance Profiling Instrumentation:**
- ✅ COMPLETED (Jan 18, 2026): Added timing instrumentation to identify performance characteristics
- ✅ Instrumented `render_with_config()` in rendering_pipeline.rs
  - Measures buffer allocation time, fractal render time, total pipeline time
- ✅ Instrumented `render_fractal()` in main.rs
  - Measures image conversion time, texture upload time, total GUI overhead
- ✅ Performance findings: 1280x720 @ 256 iterations renders in ~9-13ms (60-80 FPS capable)
- ✅ Confirmed rayon parallelization is working correctly
- ✅ No critical performance bottlenecks identified
- Note: Profiling code can be kept for future debugging or removed if desired

## Scope Notes

**Features moved to v0.1.4:**
- PNG metadata import feature (see plan_v0.1.4.md)
- Additional fractals (Tertation, etc.)

v0.1.3 focuses exclusively on cleanup, deprecation removal, and bug fixes from v0.1.2.
