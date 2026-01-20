# Plan v0.2.2 - Arbitrary Precision Depth

**Fractal & Colormap**: TBD - to be determined before release finalization

## Overview
Version 0.2.2 adds arbitrary precision arithmetic for deep zooms beyond f64 limits.

## Dependencies
- **v0.2.0 GPU Acceleration** - Need to understand GPU vs CPU trade-offs at extreme zoom
- Rendering pipeline architecture

## Main Features

### Arbitrary Depth Zoom
**Priority**: High  
**Status**: Research phase

**Description**: Enable zooms beyond f64 precision limits using arbitrary precision arithmetic.

**Implementation Approaches:**

#### Option 1: Simple f64 → Bignum Transition
- [ ] Detect when zoom exceeds f64 precision threshold
- [ ] Switch to arbitrary precision library (e.g., `rug`, `num-bigfloat`)
- [ ] Implement fractal iteration with arbitrary precision
- [ ] Handle performance trade-off (much slower than f64)
- [ ] Add UI indicator when in arbitrary precision mode
- [ ] Test with extreme zoom levels (10^100+)

#### Option 2: Perturbation Theory (Advanced)
- [ ] Research perturbation theory for deep zooms
- [ ] Implement reference point calculation
- [ ] Use deltas from reference for most pixels (f64)
- [ ] Only use arbitrary precision for reference point
- [ ] Dramatically faster than full arbitrary precision

**GPU vs CPU Considerations:**
- GPU shaders typically limited to f32/f64 precision
- Arbitrary precision requires CPU rendering
- Auto-switch to CPU when entering deep zoom mode
- May need to adjust export resolution when in CPU mode
- Perturbation theory might allow GPU for most pixels

**Benefits:**
- Enable zooms to theoretical limit (computational power)
- Discover new fractal structures
- Match capabilities of specialized deep zoom tools
- Unique feature for power users

**Technical Notes:**
- `rug` crate provides arbitrary precision via GMP
- Performance will be orders of magnitude slower
- Need clear user communication about performance trade-off
- Consider render time estimates
- May want to add "deep zoom mode" toggle