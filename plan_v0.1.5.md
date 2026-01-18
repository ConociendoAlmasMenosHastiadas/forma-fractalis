# Plan for v0.1.5 - Import/Export Features

## Overview
Version 0.1.5 will focus on round-trip functionality, allowing users to save and restore complete fractal configurations through PNG metadata.

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
