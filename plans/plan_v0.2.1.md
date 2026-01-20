# Plan v0.2.1 - Animated GIF Generation

**Fractal & Colormap**: TBD - to be determined before release finalization

## Overview
Version 0.2.1 adds animated GIF generation capabilities, enabling zoom sequences and parameter animations.

## Dependencies
- **v0.1.7 CLI Rendering** - Batch rendering foundation for frame generation
- **v0.2.0 GPU Acceleration** - Faster rendering for animation frames
- Metadata system from v0.1.6 for keyframe definitions

## Main Features

### Animated GIF Export
**Priority**: High  
**Status**: Planning phase

**Description**: Generate animated GIFs showing zoom sequences, Julia set parameter sweeps, or colormap transitions.

**Implementation:**
- [ ] Add GIF encoding capability (use `image` or `gifski` crate)
- [ ] Define animation types:
  - Zoom sequence (from point A to point B)
  - Julia parameter sweep (animate c value)
  - Colormap transition
  - Iteration count fade-in
- [ ] Add animation configuration UI
- [ ] Implement keyframe system
- [ ] Generate intermediate frames
- [ ] Support frame rate control
- [ ] Add progress indicator for long animations
- [ ] Optimize frame generation (reuse GPU state, delta rendering)

**Benefits:**
- Shareable animations on social media
- Educational demonstrations
- Artistic creations
- Marketing material for the project

**Technical Notes:**
- Leverage CLI rendering (v0.1.7) for batch frame generation
- Use GPU (v0.2.0) for faster frame rendering
- Consider `gifski` for high-quality GIF encoding
- Metadata system can store keyframes
- May need to limit resolution/frames for reasonable file sizes