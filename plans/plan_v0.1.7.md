# Plan for v0.1.7 - Command-Line Rendering

## Overview
Version 0.1.7 adds command-line rendering capabilities, building on the PNG metadata loading from v0.1.6 and the GUI architecture from v0.1.5.

**Fractal & Colormap**: TBD - to be determined before release finalization

## Dependencies
- **v0.1.5 GUI Architecture** - Grouped state pattern makes CLI integration cleaner
- **v0.1.6 PNG Loading** - Reuses metadata parsing for CLI rendering
- Metadata serialization improvements from v0.1.6

## Main Features

### Command-Line Render
**Priority**: High  
**Status**: Not started

**Description**: Provide the ability to create and render a fractal from a file passed via command line.

**Implementation:**
- [ ] Add command-line argument parsing (clap crate recommended)
- [ ] Support rendering from PNG with embedded metadata:
  ```
  forma-fractalis --render input.png --output output.png
  ```
- [ ] Support rendering from standalone JSON metadata file:
  ```
  forma-fractalis --render-json settings.json --output output.png
  ```
- [ ] Reuse PNG metadata parsing from v0.1.6
- [ ] Create headless rendering mode (no GUI)
- [ ] Leverage v0.1.5 state structs for clean CLI state management
- [ ] Support overriding parameters via CLI:
  ```
  forma-fractalis --render input.png --output output.png --width 3840 --height 2160 --iterations 8192
  ```
- [ ] Add progress indicator for long renders
- [ ] Return appropriate exit codes
- [ ] Add `--help` and `--version` flags

**Benefits:**
- Batch rendering without GUI overhead
- Automation and scripting support
- Server/cloud rendering capability
- Foundation for video generation (v0.2+)
- CI/CD integration for testing renders

**Technical Notes:**
- Use `clap` for argument parsing
- Reuse metadata deserialization from v0.1.6
- Create thin CLI wrapper around rendering pipeline
- Consider `indicatif` crate for progress bars
- Ensure headless mode doesn't require display/windowing

### Export Standalone Metadata JSON
**Priority**: Medium  
**Status**: Not started

**Description**: Allow exporting settings as JSON without rendering an image.

**Implementation:**
- [ ] Add "Export Settings" button in GUI
- [ ] Serialize current app state to JSON
- [ ] Save to .json file
- [ ] Add file picker dialog
- [ ] Validate JSON can be loaded via CLI

**Benefits:**
- Quick settings sharing (smaller than full PNG)
- Version control friendly
- Easy to edit manually
- Supports CLI rendering workflow

## Additional Features

### README Banner Images
**Priority**: Low  
**Status**: Not started

**Description**: Add visual examples to README.md to showcase the tool.

**Implementation:**
- [ ] Render several showcase fractals at high quality
- [ ] Add banner/hero image at top of README
- [ ] Add gallery section showing different fractal types
- [ ] Add gallery section showing different colormaps
- [ ] Ensure images are optimized for web display
- [ ] Consider animated GIF of zoom sequence (if feasible)

**Benefits:**
- Better first impression
- Visual demonstration of capabilities
- Showcases colormap variety
- Attracts more users/contributors


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
- [ ] Add to FractalType enum in `fractals/mod.rs`
- [ ] Add GUI controls for k parameter
- [ ] Test rendering and convergence
- [ ] Document formula and interesting regions

**Note:** This is lower priority than CLI features. Consider moving to v0.1.8 if time is limited.

## Architecture Notes

### Integration with v0.1.5/v0.1.6 Improvements

The CLI rendering leverages several architectural improvements:

1. **Grouped State Pattern (v0.1.5):**
   - CLI can construct state structs directly
   - No need to replicate complex initialization logic
   - Easy to validate state before rendering

2. **Metadata Parsing (v0.1.6):**
   - Reuse PNG metadata parser for CLI input
   - Consistent deserialization logic
   - Same validation rules apply

3. **Headless Rendering:**
   - Rendering pipeline already separate from GUI
   - Just need entry point that doesn't initialize egui
   - State structs make this clean

**Example CLI Architecture:**
```rust
// CLI entry point
fn main_cli(args: CliArgs) -> Result<()> {
    // Load metadata from PNG or JSON
    let metadata = load_metadata(&args.input)?;
    
    // Construct state structs (reusing v0.1.5 patterns)
    let fractal_state = metadata.into_fractal_state();
    let view_state = metadata.into_view_state();
    let color_state = metadata.into_color_state();
    let export_state = metadata.into_export_state();
    
    // Override with CLI args if provided
    if let Some(width) = args.width {
        export_state.width = width;
    }
    // ... more overrides
    
    // Render (no GUI needed)
    let image = render_fractal(
        &fractal_state,
        &view_state,
        &color_state,
        &export_state,
    )?;
    
    // Save
    save_png(&args.output, &image, &metadata)?;
    
    Ok(())
}
```

## Testing and Validation

**Implementation:**
- [ ] Test CLI rendering produces identical output to GUI rendering
- [ ] Test all parameter overrides work correctly
- [ ] Test error handling (missing files, invalid JSON, etc.)
- [ ] Test progress indicators work on different terminals
- [ ] Benchmark CLI vs GUI rendering performance
- [ ] Test on systems without display/X11
- [ ] Add integration tests for CLI workflows

**Success Criteria:**
- CLI produces pixel-perfect match to GUI rendering
- All error cases handled gracefully
- Help documentation is clear and complete
- Exit codes follow conventions

## Progress Tracking

### Implementation Notes
- *Track progress here as implementation proceeds*

### Blockers
- None currently (v0.1.5 and v0.1.6 must complete first)

### Testing Results
- *Record test results here*

## Release Criteria
- [ ] CLI rendering from PNG metadata works
- [ ] CLI rendering from JSON metadata works
- [ ] Parameter overrides work correctly
- [ ] Progress indicators implemented
- [ ] Help documentation complete
- [ ] All tests pass
- [ ] Documentation updated (README.md with CLI examples)
- [ ] Benchmarks show CLI performance is good
- [ ] Error handling is robust