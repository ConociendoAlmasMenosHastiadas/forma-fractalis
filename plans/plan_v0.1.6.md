# Plan for v0.1.6 - Import/Export Features

## Overview
Version 0.1.6 focuses on completing the round-trip functionality with Load from PNG and preparing the foundation for command-line rendering in v0.1.7.

**Fractal & Colormap**: TBD - to be determined before release finalization

## Priority Order
1. **Load from PNG Metadata** - Major feature for round-trip functionality
2. **Improve metadata serialization** - Prepare for command-line usage
3. **Testing and validation** - Ensure robust import/export

## Dependencies
- **v0.1.5 GUI Architecture Refactoring** must be complete - the new grouped state pattern (ViewState, FractalState, ColorState, etc.) makes loading state much cleaner
- PNG metadata export (completed in v0.1.2)

## Main Features

### Load from PNG Metadata
**Priority**: High  
**Status**: Not started  
**Dependencies**: v0.1.5 GUI refactoring, PNG metadata export (v0.1.2)

**Description**: Add "Load from PNG" button to import all settings from exported images with embedded metadata.

**Implementation:**
- [ ] Add "Load from PNG" button in GUI (near export section)
- [ ] Parse PNG tEXt chunks using png crate
- [ ] Deserialize metadata JSON from tEXt chunks
- [ ] Apply all loaded settings using new state structs from v0.1.5:
  - Switch to correct fractal type (FractalState)
  - Load fractal parameters (Julia c values, etc.) (FractalState)
  - Set view coordinates and zoom (ViewState)
  - Restore colormap (including custom color stops) (ColorState)
  - Apply color modulation settings (period, interior color, log scale) (ColorState)
  - Set export settings (filter, supersample, scale) (ExportState)
  - Update text inputs (InputState)
- [ ] Handle missing/invalid metadata gracefully
- [ ] Show success/error message to user
- [ ] Add file picker dialog for PNG selection (use rfd::FileDialog)
- [ ] Trigger redraw after loading

**Benefits:**
- Complete round-trip: Export → Load → Exact reproduction
- Share discoveries with full settings embedded
- Easy iteration on exported images
- No manual settings transcription needed
- Foundation for command-line rendering (v0.1.7)

**Technical Notes:**
- Use `rfd::FileDialog` for PNG selection
- Validate fractal type and parameters before applying
- Consider adding "Preview Metadata" option to show settings before loading
- With v0.1.5's state grouping, loading becomes:
  ```rust
  // Parse metadata
  let metadata: FractalMetadata = parse_png_metadata(&png_path)?;
  
  // Apply to grouped state
  app.fractal = metadata.into_fractal_state();
  app.view = metadata.into_view_state();
  app.color = metadata.into_color_state();
  app.export = metadata.into_export_state();
  app.input = metadata.into_input_state();
  ```

**Integration with GUI Architecture:**
The v0.1.5 refactoring makes this much cleaner:
- Loading function takes `&mut FractalApp` instead of 15+ individual parameters
- State validation can be encapsulated in state struct methods
- Error handling is centralized
- Easy to add new state fields without changing load logic

### Metadata Serialization Improvements
**Priority**: Medium  
**Status**: Enhancement of existing system  

**Description**: Improve metadata structure to better support command-line rendering and future features.

**Implementation:**
- [ ] Review current metadata JSON structure in `export.rs`
- [ ] Ensure all state is serializable/deserializable
- [ ] Add version field to metadata for future compatibility
- [ ] Add optional "description" or "name" field for user annotations
- [ ] Consider adding timestamp for when fractal was created
- [ ] Test round-trip serialization for all fractal types
- [ ] Document metadata format in dedicated file (e.g., METADATA_FORMAT.md)

**Benefits:**
- Better foundation for command-line tools
- Future-proof metadata format
- User-friendly fractal organization
- Easier debugging and validation

### Testing and Validation
**Priority**: High  
**Status**: Not started  

**Implementation:**
- [ ] Add tests for PNG metadata parsing
- [ ] Test all fractal types can be loaded correctly
- [ ] Test custom colormap loading
- [ ] Test edge cases (missing fields, wrong types, corrupted data)
- [ ] Test loading into fresh application state
- [ ] Test loading while other fractal is displayed (state replacement)
- [ ] Add integration test that exports then loads and compares results
- [ ] Test with PNGs that have no metadata (should gracefully decline)
- [ ] Test with PNGs that have partial/corrupted metadata

**Success Criteria:**
- Load any exported PNG and reproduce exact visual result
- Graceful handling of all error cases with user-friendly messages
- No crashes or panics from malformed metadata
- All tests pass

## Future Considerations
**NOTE:** These will move to v0.1.7 or later

- Command-line rendering from PNG files (v0.1.7 target)
- Support for batch loading multiple PNGs
- Playlist/gallery mode to cycle through saved fractals
- Export/import of just settings (JSON without image)
- Drag-and-drop PNG loading
- "Recent fractals" menu

## Progress Tracking

### Implementation Notes
- *Track progress here as implementation proceeds*

### Blockers
- None currently (v0.1.5 must complete first)

### Testing Results
- *Record test results here*

## Release Criteria
- [ ] Load from PNG feature fully implemented and tested
- [ ] All fractal types can be loaded and rendered correctly
- [ ] Custom colormaps restore properly
- [ ] All state (view, fractal, color, export) restores correctly
- [ ] Error handling is robust and user-friendly
- [ ] Documentation updated (README.md, user guide if exists)
- [ ] No regressions in existing export functionality
- [ ] Code follows patterns established in v0.1.5 (grouped state)
