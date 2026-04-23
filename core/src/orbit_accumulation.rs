//! Orbit Accumulation Rendering Backend
//!
//! Shared infrastructure for fractals that render via orbit density (chaos game,
//! Buddhabrot, etc.) rather than per-pixel escape-time iteration. Provides:
//!
//! - [`DensityBuffer`]: Histogram buffer that maps complex-plane coordinates to
//!   pixel bins and accumulates visit counts.
//! - [`AtomicDensityBuffer`]: Thread-safe histogram using `AtomicU64` per pixel.
//!   Used for large resolutions where per-thread `DensityBuffer` copies would OOM.
//! - [`OrbitTarget`]: Trait abstracting over mutable and atomic density targets,
//!   allowing `Fractal::accumulate_orbits` to work with either buffer type.
//! - [`complex_sqrt`]: Principal complex square root utility.
//! - [`compute_orbit_density`]: High-level entry point that calls a fractal's
//!   `accumulate_orbits()` method, parallelises across sub-orbits, merges
//!   density buffers, and normalises to `Vec<u32>` compatible with the standard
//!   coloring pipeline. Automatically switches to the atomic path when buffer
//!   size exceeds [`ATOMIC_THRESHOLD_BYTES`].
//!
//! The output of this module (`Vec<u32>` in `[0, max_iter-1]`) is
//! indistinguishable from escape-time iteration counts as far as the coloring
//! pipeline is concerned. `apply_colors_from_cache()`, colormaps, period
//! modulation, interior color, log scale, and export all work unchanged.

use num_complex::Complex64;
use rayon::prelude::*;
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::fractals::{Fractal, FractalView};
use std::collections::HashMap;
use astro_float::{BigFloat, RoundingMode};

/// Number of fixed sub-orbits used for parallel decomposition.
/// Output is deterministic for a given seed regardless of thread count because
/// the sub-orbit count and per-sub-orbit seeds are fixed.
/// Increase if you see banding artifacts from insufficient orbit coverage.
pub const SUB_ORBIT_COUNT: u64 = 64;

/// Golden-ratio constant used for seed mixing (Knuth multiplicative hash).
const GOLDEN_RATIO: u64 = 0x9e3779b97f4a7c15;

/// When a single DensityBuffer would exceed this size (in bytes), the parallel
/// dispatch switches from fold/reduce (one buffer per thread) to the atomic
/// shared-buffer path (one `AtomicDensityBuffer` shared by all threads).
///
/// 500 MB — at 8 bytes/pixel this is ~62.5 million pixels (~7900x7900).
/// Preview resolutions stay on the fast fold/reduce path; 4x supersampled 4K
/// exports (15360x8640 = 132M pixels = 1 GB) use the atomic path.
pub const ATOMIC_THRESHOLD_BYTES: usize = 500 * 1024 * 1024;

// ── OrbitTarget trait ──────────────────────────────────────────────────────

/// Abstraction over density buffer types for orbit accumulation.
///
/// Both [`LocalDensityTarget`] (single-thread, Cell-based) and
/// [`AtomicDensityBuffer`] (multi-thread, AtomicU64-based) implement this
/// trait. `Fractal::accumulate_orbits` takes `&dyn OrbitTarget` so the same
/// orbit logic works with either buffer strategy.
pub trait OrbitTarget {
    /// Map a complex-plane coordinate to a pixel and increment its count.
    /// If the point falls outside the viewport, it is silently ignored.
    fn increment(&self, z: Complex64, view: &FractalView);
}

// ── DensityBuffer ──────────────────────────────────────────────────────────

/// A 2D histogram that accumulates orbit visit counts for density-based rendering.
///
/// Each cell stores a `u64` hit count. Use [`increment`](DensityBuffer::increment)
/// to record orbit visits and [`normalize`](DensityBuffer::normalize) to convert
/// to a `Vec<u32>` suitable for the coloring pipeline.
pub struct DensityBuffer {
    data: Vec<u64>,
    width: u32,
    height: u32,
}

impl DensityBuffer {
    /// Allocate a zeroed density buffer for the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            data: vec![0u64; (width as usize) * (height as usize)],
            width,
            height,
        }
    }

    /// Create a density buffer from raw u32 counts (e.g., from GPU readback).
    pub fn from_raw(width: u32, height: u32, raw: Vec<u32>) -> Self {
        Self {
            data: raw.into_iter().map(|v| v as u64).collect(),
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Map a complex-plane coordinate to a pixel and increment its count.
    /// If the point falls outside the viewport, it is silently ignored.
    #[inline]
    pub fn increment(&mut self, z: Complex64, view: &FractalView) {
        if let Some((px, py)) = complex_to_screen(z.re, z.im, view) {
            let idx = (py as usize) * (self.width as usize) + (px as usize);
            // Safety: complex_to_screen guarantees px < width and py < height
            self.data[idx] += 1;
        }
    }

    /// Increment the density count at the given pixel coordinates.
    /// Caller must ensure `px < width` and `py < height`.
    #[inline]
    pub fn increment_at(&mut self, px: u32, py: u32) {
        let idx = (py as usize) * (self.width as usize) + (px as usize);
        self.data[idx] += 1;
    }

    /// Element-wise merge another buffer into this one (for combining sub-orbits).
    /// Panics if dimensions differ.
    pub fn merge(&mut self, other: &DensityBuffer) {
        assert_eq!(self.width, other.width, "DensityBuffer merge: width mismatch");
        assert_eq!(self.height, other.height, "DensityBuffer merge: height mismatch");
        for (dst, src) in self.data.iter_mut().zip(other.data.iter()) {
            *dst += *src;
        }
    }

    /// Normalise the histogram to `[0, max_iter - 1]` as `Vec<u32>`.
    ///
    /// When `use_log` is true, applies `log(1 + count) / log(1 + max_count)`
    /// to compress the extreme dynamic range typical of orbit-density images.
    /// Without log normalisation the image is effectively binary.
    pub fn normalize(&self, max_iter: u32, use_log: bool) -> Vec<u32> {
        let max_count = self.data.iter().copied().max().unwrap_or(0);
        if max_count == 0 {
            return vec![0u32; self.data.len()];
        }

        let scale = (max_iter - 1) as f64;

        if use_log {
            let log_max = (1.0 + max_count as f64).ln();
            self.data
                .iter()
                .map(|&count| {
                    let t = (1.0 + count as f64).ln() / log_max;
                    (t * scale).round() as u32
                })
                .collect()
        } else {
            let max_f = max_count as f64;
            self.data
                .iter()
                .map(|&count| {
                    let t = count as f64 / max_f;
                    (t * scale).round() as u32
                })
                .collect()
        }
    }
}

// ── LocalDensityTarget ─────────────────────────────────────────────────────

/// Single-thread density target using `Cell<u64>` for interior mutability.
///
/// Used in the fold/reduce parallel path where each rayon thread owns one
/// target exclusively. `Cell` provides `&self` mutation without atomics.
/// NOT `Sync` (cannot be shared between threads) — this is intentional.
pub struct LocalDensityTarget {
    data: Vec<Cell<u64>>,
    width: u32,
    height: u32,
}

impl LocalDensityTarget {
    /// Allocate a zeroed local density target for the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            data: vec![Cell::new(0u64); (width as usize) * (height as usize)],
            width,
            height,
        }
    }

    /// Element-wise merge another target into this one.
    pub fn merge(&self, other: &LocalDensityTarget) {
        assert_eq!(self.width, other.width, "LocalDensityTarget merge: width mismatch");
        assert_eq!(self.height, other.height, "LocalDensityTarget merge: height mismatch");
        for (dst, src) in self.data.iter().zip(other.data.iter()) {
            dst.set(dst.get() + src.get());
        }
    }

    /// Convert into a `DensityBuffer` for normalisation.
    pub fn into_density_buffer(self) -> DensityBuffer {
        DensityBuffer {
            data: self.data.into_iter().map(|c| c.get()).collect(),
            width: self.width,
            height: self.height,
        }
    }
}

impl OrbitTarget for LocalDensityTarget {
    #[inline]
    fn increment(&self, z: Complex64, view: &FractalView) {
        if let Some((px, py)) = complex_to_screen(z.re, z.im, view) {
            let idx = (py as usize) * (self.width as usize) + (px as usize);
            self.data[idx].set(self.data[idx].get() + 1);
        }
    }
}

// ── AtomicDensityBuffer ────────────────────────────────────────────────────

/// Thread-safe density buffer using `AtomicU64` per pixel.
///
/// All threads write to the same buffer via `fetch_add` with `Relaxed` ordering.
/// Peak memory is ONE buffer regardless of thread count. Used when the buffer
/// size exceeds [`ATOMIC_THRESHOLD_BYTES`] to avoid OOM in supersampled exports.
///
/// `Relaxed` ordering is sufficient because all reads (in `normalize()`) happen
/// strictly after all writes complete (rayon's `for_each` synchronises).
pub struct AtomicDensityBuffer {
    data: Vec<AtomicU64>,
    width: u32,
    height: u32,
}

// AtomicU64 is Send + Sync, so AtomicDensityBuffer is too.
// Explicit impl not needed (derived automatically), but stating for clarity.

impl AtomicDensityBuffer {
    /// Allocate a zeroed atomic density buffer for the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let len = (width as usize) * (height as usize);
        let mut data = Vec::with_capacity(len);
        for _ in 0..len {
            data.push(AtomicU64::new(0));
        }
        Self { data, width, height }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Normalise the histogram to `[0, max_iter - 1]` as `Vec<u32>`.
    ///
    /// Must be called only after all parallel writes have completed (e.g. after
    /// rayon `for_each` returns). Uses `Relaxed` loads — safe because rayon
    /// provides the happens-before ordering.
    pub fn normalize(&self, max_iter: u32, use_log: bool) -> Vec<u32> {
        let max_count = self.data.iter()
            .map(|a| a.load(Ordering::Relaxed))
            .max()
            .unwrap_or(0);
        if max_count == 0 {
            return vec![0u32; self.data.len()];
        }

        let scale = (max_iter - 1) as f64;

        if use_log {
            let log_max = (1.0 + max_count as f64).ln();
            self.data
                .iter()
                .map(|a| {
                    let count = a.load(Ordering::Relaxed);
                    let t = (1.0 + count as f64).ln() / log_max;
                    (t * scale).round() as u32
                })
                .collect()
        } else {
            let max_f = max_count as f64;
            self.data
                .iter()
                .map(|a| {
                    let count = a.load(Ordering::Relaxed);
                    let t = count as f64 / max_f;
                    (t * scale).round() as u32
                })
                .collect()
        }
    }
}

impl OrbitTarget for AtomicDensityBuffer {
    #[inline]
    fn increment(&self, z: Complex64, view: &FractalView) {
        if let Some((px, py)) = complex_to_screen(z.re, z.im, view) {
            let idx = (py as usize) * (self.width as usize) + (px as usize);
            self.data[idx].fetch_add(1, Ordering::Relaxed);
        }
    }
}

// ── Coordinate mapping ─────────────────────────────────────────────────────

/// Inverse of `FractalView::screen_to_complex()`.
/// Maps a complex-plane point back to pixel coordinates.
/// Returns `None` if the point falls outside the viewport.
#[inline]
pub fn complex_to_screen(real: f64, imag: f64, view: &FractalView) -> Option<(u32, u32)> {
    let aspect_ratio = view.width as f64 / view.height as f64;
    let scale = 3.5 / view.zoom;

    // Invert: real = center_x + (x - w/2) * scale / w * aspect
    //         x = (real - center_x) * w / (scale * aspect) + w/2
    let px_f = (real - view.center_x) * view.width as f64 / (scale * aspect_ratio)
        + view.width as f64 / 2.0;
    let py_f = (imag - view.center_y) * view.height as f64 / scale
        + view.height as f64 / 2.0;

    if px_f < 0.0 || py_f < 0.0 {
        return None;
    }
    let px = px_f as u32;
    let py = py_f as u32;
    if px >= view.width || py >= view.height {
        return None;
    }
    Some((px, py))
}

// ── Complex sqrt ───────────────────────────────────────────────────────────

/// Principal square root of a complex number.
///
/// `sqrt(a + bi) = sqrt(r) * (cos(theta/2) + i*sin(theta/2))`
/// where `r = |w|` and `theta = atan2(b, a)`.
#[inline]
pub fn complex_sqrt(w: Complex64) -> Complex64 {
    let r = w.norm();
    if r < 1e-300 {
        return Complex64::new(0.0, 0.0);
    }
    let sqrt_r = r.sqrt();
    let half_theta = w.arg() / 2.0;
    Complex64::new(sqrt_r * half_theta.cos(), sqrt_r * half_theta.sin())
}

// ── Hi-precision helpers ───────────────────────────────────────────────────

/// Convert a BigFloat to f64 via Display formatting.
///
/// BigFloat has no `to_f64()` method (as of astro-float 0.9).
/// This is acceptable in the hiprec orbit path where BigFloat arithmetic
/// already dominates the runtime.
fn bigfloat_to_f64(bf: &BigFloat) -> f64 {
    if bf.is_nan() || bf.is_inf() {
        return f64::NAN;
    }
    if bf.is_zero() {
        return 0.0;
    }
    format!("{}", bf).parse::<f64>().unwrap_or(f64::NAN)
}

/// Hi-precision complex square root using the algebraic identity:
///
/// `sqrt(a + bi) = sqrt((r+a)/2) + i * sign(b) * sqrt((r-a)/2)`
/// where `r = sqrt(a^2 + b^2)`.
///
/// This avoids transcendental functions (no trig needed) and uses only
/// BigFloat `sqrt`, `add`, `sub`, `mul`, `div`.
pub fn complex_sqrt_hiprec(w_re: &BigFloat, w_im: &BigFloat, bits: u32) -> (BigFloat, BigFloat) {
    let p = bits as usize;
    let rm = RoundingMode::ToEven;
    let two = BigFloat::from_f64(2.0, p);

    let re_sq = w_re.mul(w_re, p, rm);
    let im_sq = w_im.mul(w_im, p, rm);
    let norm_sq = re_sq.add(&im_sq, p, rm);

    // r = sqrt(|w|^2)
    let r = norm_sq.sqrt(p, rm);

    if r.is_zero() || r.is_nan() {
        let zero = BigFloat::from_f64(0.0, p);
        return (zero.clone(), zero);
    }

    // u = sqrt((r + a) / 2)
    let r_plus_a = r.add(w_re, p, rm);
    let half_rpa = r_plus_a.div(&two, p, rm);
    let u = half_rpa.sqrt(p, rm);

    // v = sign(b) * sqrt((r - a) / 2)
    let r_minus_a = r.sub(w_re, p, rm);
    let half_rma = r_minus_a.div(&two, p, rm);
    let v_abs = half_rma.sqrt(p, rm);

    let v = if w_im.is_negative() {
        v_abs.neg()
    } else {
        v_abs
    };

    (u, v)
}

/// Hi-precision inverse coordinate mapping.
///
/// Computes the critical `(z - center)` subtraction in BigFloat to avoid
/// catastrophic cancellation at extreme zoom levels, then converts the
/// small offset to f64 for the final pixel calculation.
///
/// The offset is `O(3.5 / zoom / width)` per pixel, so f64 has sufficient
/// precision for the multiplication/division step as long as the subtraction
/// is done in BigFloat.
pub fn complex_to_screen_hiprec(
    real: &BigFloat,
    imag: &BigFloat,
    view: &FractalView,
    bits: u32,
) -> Option<(u32, u32)> {
    let p = bits as usize;
    let rm = RoundingMode::ToEven;

    let center_x = BigFloat::from_f64(view.center_x, p);
    let center_y = BigFloat::from_f64(view.center_y, p);

    // Compute offsets in BigFloat — this is where precision matters
    let dx = real.sub(&center_x, p, rm);
    let dy = imag.sub(&center_y, p, rm);

    // Convert the small offsets to f64
    let dx_f64 = bigfloat_to_f64(&dx);
    let dy_f64 = bigfloat_to_f64(&dy);

    if !dx_f64.is_finite() || !dy_f64.is_finite() {
        return None;
    }

    // Pixel mapping in f64 (offset is small, f64 is fine for mul/div)
    let scale = 3.5 / view.zoom;
    let aspect_ratio = view.width as f64 / view.height as f64;

    let px_f = dx_f64 * view.width as f64 / (scale * aspect_ratio) + view.width as f64 / 2.0;
    let py_f = dy_f64 * view.height as f64 / scale + view.height as f64 / 2.0;

    if px_f < 0.0 || py_f < 0.0 {
        return None;
    }
    let px = px_f as u32;
    let py = py_f as u32;
    if px >= view.width || py >= view.height {
        return None;
    }
    Some((px, py))
}

// ── Seed mixing ────────────────────────────────────────────────────────────

/// Derive a deterministic sub-orbit seed from a master seed and an index.
#[inline]
pub fn derive_sub_seed(master_seed: u64, index: u64) -> u64 {
    let s = master_seed
        .wrapping_mul(GOLDEN_RATIO)
        .wrapping_add(index.wrapping_mul(0x6c62272e07bb0142));
    // Ensure non-zero (xorshift64 requires it)
    if s == 0 { 0xdeadbeefcafebabe } else { s }
}

// ── High-level entry point ─────────────────────────────────────────────────

/// Compute orbit-density data for a fractal that uses orbit accumulation.
///
/// Parallelises across [`SUB_ORBIT_COUNT`] independent sub-orbits and
/// normalises to `Vec<u32>` compatible with `apply_colors_from_cache()`.
///
/// **Path selection** (based on buffer size):
/// - Small buffers (< [`ATOMIC_THRESHOLD_BYTES`]): fold/reduce with per-thread
///   `LocalDensityTarget`. Faster due to no atomic contention.
/// - Large buffers (>= threshold): single shared `AtomicDensityBuffer` with
///   `par_iter().for_each()`. Uses ~2 GB instead of ~20 GB at 4x supersampled 4K.
///
/// `max_threads`: passed through to rayon pool sizing (0 = use all available).
pub fn compute_orbit_density(
    view: &FractalView,
    fractal: &dyn Fractal,
    params: &HashMap<String, f64>,
    max_iter: u32,
    max_threads: usize,
) -> Vec<u32> {
    assert!(
        fractal.uses_orbit_accumulation(),
        "compute_orbit_density called on '{}' which does not use orbit accumulation",
        fractal.name()
    );

    let total_samples = params.get("samples").copied().unwrap_or(5_000_000.0) as u64;
    let use_log = params.get("use_log_density").copied().unwrap_or(1.0) > 0.5;
    let master_seed = params.get("seed").copied().unwrap_or(0.0) as u64;

    let k = SUB_ORBIT_COUNT;
    let w = view.width;
    let h = view.height;

    // Pre-generate all sub-orbit work items: (sub_seed, num_steps)
    let work_items: Vec<(u64, u64)> = (0..k)
        .map(|i| {
            let sub_seed = derive_sub_seed(master_seed, i);
            let steps = total_samples / k + if i < (total_samples % k) { 1 } else { 0 };
            (sub_seed, steps)
        })
        .collect();

    let buffer_bytes = (w as usize) * (h as usize) * std::mem::size_of::<u64>();

    if buffer_bytes >= ATOMIC_THRESHOLD_BYTES {
        // ── Atomic path: one shared AtomicDensityBuffer ────────────────
        let atomic_buf = AtomicDensityBuffer::new(w, h);

        let compute_atomic = || {
            work_items.par_iter().for_each(|&(sub_seed, steps)| {
                let mut sub_params = params.clone();
                sub_params.insert("seed".to_string(), sub_seed as f64);
                sub_params.insert("samples".to_string(), steps as f64);
                fractal.accumulate_orbits(&atomic_buf, view, &sub_params);
            });
        };

        let global_count = rayon::current_num_threads();
        if max_threads > 0 && max_threads < global_count {
            match rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build()
            {
                Ok(pool) => pool.install(compute_atomic),
                Err(_) => compute_atomic(),
            }
        } else {
            compute_atomic();
        }

        atomic_buf.normalize(max_iter, use_log)
    } else {
        // ── Fold/reduce path: per-thread LocalDensityTarget ────────────
        let compute = || -> DensityBuffer {
            work_items
                .par_iter()
                .fold(
                    || LocalDensityTarget::new(w, h),
                    |acc, &(sub_seed, steps)| {
                        let mut sub_params = params.clone();
                        sub_params.insert("seed".to_string(), sub_seed as f64);
                        sub_params.insert("samples".to_string(), steps as f64);
                        fractal.accumulate_orbits(&acc, view, &sub_params);
                        acc
                    },
                )
                .reduce(
                    || LocalDensityTarget::new(w, h),
                    |a, b| {
                        a.merge(&b);
                        a
                    },
                )
                .into_density_buffer()
        };

        let global_count = rayon::current_num_threads();
        let merged = if max_threads > 0 && max_threads < global_count {
            match rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build()
            {
                Ok(pool) => pool.install(compute),
                Err(_) => compute(),
            }
        } else {
            compute()
        };

        merged.normalize(max_iter, use_log)
    }
}

// ── Hi-precision orbit density entry point ─────────────────────────────────

/// Hi-precision version of [`compute_orbit_density`].
///
/// Uses BigFloat arithmetic for the orbit computation and coordinate mapping.
/// The fractal must implement `accumulate_orbits_hiprec()`.
///
/// Uses the fold/reduce pattern with `DensityBuffer` (not atomic path).
/// Hi-prec is inherently slow, so users will use moderate resolutions where
/// fold/reduce is safe (no OOM risk).
pub fn compute_orbit_density_hiprec(
    view: &FractalView,
    fractal: &dyn Fractal,
    params: &HashMap<String, f64>,
    max_iter: u32,
    max_threads: usize,
    bits: u32,
) -> Result<Vec<u32>, String> {
    if !fractal.uses_orbit_accumulation() {
        return Err(format!(
            "compute_orbit_density_hiprec called on '{}' which does not use orbit accumulation",
            fractal.name()
        ));
    }
    if !fractal.supports_hiprec() {
        return Err(format!(
            "Fractal '{}' does not support hi-precision rendering",
            fractal.name()
        ));
    }

    let total_samples = params.get("samples").copied().unwrap_or(5_000_000.0) as u64;
    let use_log = params.get("use_log_density").copied().unwrap_or(1.0) > 0.5;
    let master_seed = params.get("seed").copied().unwrap_or(0.0) as u64;

    let k = SUB_ORBIT_COUNT;
    let w = view.width;
    let h = view.height;

    let work_items: Vec<(u64, u64)> = (0..k)
        .map(|i| {
            let sub_seed = derive_sub_seed(master_seed, i);
            let steps = total_samples / k + if i < (total_samples % k) { 1 } else { 0 };
            (sub_seed, steps)
        })
        .collect();

    let compute = || -> DensityBuffer {
        work_items
            .par_iter()
            .fold(
                || DensityBuffer::new(w, h),
                |mut acc, &(sub_seed, steps)| {
                    let mut sub_params = params.clone();
                    sub_params.insert("seed".to_string(), sub_seed as f64);
                    sub_params.insert("samples".to_string(), steps as f64);
                    fractal.accumulate_orbits_hiprec(&mut acc, view, &sub_params, bits);
                    acc
                },
            )
            .reduce(
                || DensityBuffer::new(w, h),
                |mut a, b| {
                    a.merge(&b);
                    a
                },
            )
    };

    let global_count = rayon::current_num_threads();
    let merged = if max_threads > 0 && max_threads < global_count {
        match rayon::ThreadPoolBuilder::new()
            .num_threads(max_threads)
            .build()
        {
            Ok(pool) => pool.install(compute),
            Err(_) => compute(),
        }
    } else {
        compute()
    };

    Ok(merged.normalize(max_iter, use_log))
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fractals::FractalView;

    #[test]
    fn test_density_buffer_dimensions() {
        let buf = DensityBuffer::new(100, 80);
        assert_eq!(buf.width(), 100);
        assert_eq!(buf.height(), 80);
        assert_eq!(buf.data.len(), 100 * 80);
    }

    #[test]
    fn test_increment_in_bounds() {
        let view = FractalView::new(100, 100);
        // Default view: center=(0,0), zoom=1.0, scale=3.5
        // The center of the image (pixel 50,50) corresponds to (0,0)
        let mut buf = DensityBuffer::new(100, 100);
        buf.increment(Complex64::new(0.0, 0.0), &view);
        // Pixel at center should have count 1
        let center_idx = 50 * 100 + 50;
        assert_eq!(buf.data[center_idx], 1);
        // Increment again
        buf.increment(Complex64::new(0.0, 0.0), &view);
        assert_eq!(buf.data[center_idx], 2);
    }

    #[test]
    fn test_increment_out_of_bounds() {
        let view = FractalView::new(100, 100);
        let mut buf = DensityBuffer::new(100, 100);
        // Far outside the viewport — should not panic
        buf.increment(Complex64::new(100.0, 100.0), &view);
        buf.increment(Complex64::new(-100.0, -100.0), &view);
        // Total should be zero
        let total: u64 = buf.data.iter().sum();
        assert_eq!(total, 0);
    }

    #[test]
    fn test_normalize_range() {
        let mut buf = DensityBuffer::new(10, 10);
        // Set some varied counts
        buf.data[0] = 0;
        buf.data[1] = 50;
        buf.data[2] = 100;
        buf.data[3] = 1000;

        let max_iter = 256;
        let result = buf.normalize(max_iter, false);
        assert_eq!(result.len(), 100);
        for &v in &result {
            assert!(v < max_iter, "normalized value {} >= max_iter {}", v, max_iter);
        }
        // Max count pixel should map to max_iter - 1
        assert_eq!(result[3], 255);
        // Zero-count pixel should map to 0
        assert_eq!(result[0], 0);
    }

    #[test]
    fn test_normalize_log_vs_linear() {
        let mut buf = DensityBuffer::new(10, 10);
        buf.data[0] = 1;
        buf.data[1] = 10;
        buf.data[2] = 100;
        buf.data[3] = 10000;

        let linear = buf.normalize(256, false);
        let log = buf.normalize(256, true);

        // Log should compress the range: low-count pixels get higher values
        // than linear, high-count pixels get relatively lower
        assert!(
            log[0] > linear[0],
            "log normalization should boost low-count pixels: log={} linear={}",
            log[0], linear[0]
        );
    }

    #[test]
    fn test_normalize_all_zero() {
        let buf = DensityBuffer::new(10, 10);
        let result = buf.normalize(256, true);
        assert!(result.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_merge() {
        let mut a = DensityBuffer::new(10, 10);
        let mut b = DensityBuffer::new(10, 10);
        a.data[0] = 5;
        a.data[1] = 10;
        b.data[0] = 3;
        b.data[1] = 7;
        a.merge(&b);
        assert_eq!(a.data[0], 8);
        assert_eq!(a.data[1], 17);
    }

    #[test]
    #[should_panic(expected = "width mismatch")]
    fn test_merge_dimension_mismatch() {
        let mut a = DensityBuffer::new(10, 10);
        let b = DensityBuffer::new(20, 10);
        a.merge(&b);
    }

    #[test]
    fn test_complex_sqrt_positive_real() {
        let w = Complex64::new(4.0, 0.0);
        let s = complex_sqrt(w);
        assert!((s.re - 2.0).abs() < 1e-10);
        assert!(s.im.abs() < 1e-10);
    }

    #[test]
    fn test_complex_sqrt_negative_real() {
        let w = Complex64::new(-1.0, 0.0);
        let s = complex_sqrt(w);
        // sqrt(-1) = i
        assert!(s.re.abs() < 1e-10);
        assert!((s.im - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_complex_sqrt_general() {
        let w = Complex64::new(3.0, 4.0);
        let s = complex_sqrt(w);
        // Verify s*s ≈ w
        let product = s * s;
        assert!((product.re - w.re).abs() < 1e-10);
        assert!((product.im - w.im).abs() < 1e-10);
    }

    #[test]
    fn test_complex_sqrt_zero() {
        let s = complex_sqrt(Complex64::new(0.0, 0.0));
        assert!(s.re.abs() < 1e-10);
        assert!(s.im.abs() < 1e-10);
    }

    #[test]
    fn test_derive_sub_seed_deterministic() {
        let s1 = derive_sub_seed(42, 0);
        let s2 = derive_sub_seed(42, 0);
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_derive_sub_seed_distinct() {
        let seeds: Vec<u64> = (0..64).map(|i| derive_sub_seed(42, i)).collect();
        // All 64 should be unique
        let mut deduped = seeds.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(deduped.len(), 64, "sub-orbit seeds should all be distinct");
    }

    #[test]
    fn test_derive_sub_seed_nonzero() {
        // Even tricky inputs should never produce 0
        for master in [0u64, 1, u64::MAX, 0xdeadbeef] {
            for idx in 0..64 {
                assert_ne!(derive_sub_seed(master, idx), 0);
            }
        }
    }

    #[test]
    fn test_complex_to_screen_roundtrip() {
        let view = FractalView {
            center_x: -0.5,
            center_y: 0.3,
            zoom: 2.0,
            width: 800,
            height: 600,
            parameters: HashMap::new(),
        };
        // Pick a pixel, convert to complex, convert back to pixel
        let (re, im) = view.screen_to_complex(400, 300);
        let (px, py) = complex_to_screen(re, im, &view).expect("should be in bounds");
        assert_eq!(px, 400);
        assert_eq!(py, 300);
    }

    #[test]
    fn test_complex_to_screen_corners() {
        let view = FractalView::new(100, 100);
        // Top-left pixel (0,0)
        let (re, im) = view.screen_to_complex(0, 0);
        let result = complex_to_screen(re, im, &view);
        assert!(result.is_some());
        let (px, py) = result.unwrap();
        assert_eq!(px, 0);
        assert_eq!(py, 0);
    }

    // ── LocalDensityTarget tests ───────────────────────────────────────

    #[test]
    fn test_local_density_target_increment() {
        let view = FractalView::new(100, 100);
        let target = LocalDensityTarget::new(100, 100);
        target.increment(Complex64::new(0.0, 0.0), &view);
        target.increment(Complex64::new(0.0, 0.0), &view);
        let buf = target.into_density_buffer();
        let center_idx = 50 * 100 + 50;
        assert_eq!(buf.data[center_idx], 2);
    }

    #[test]
    fn test_local_density_target_out_of_bounds() {
        let view = FractalView::new(100, 100);
        let target = LocalDensityTarget::new(100, 100);
        target.increment(Complex64::new(100.0, 100.0), &view);
        let buf = target.into_density_buffer();
        let total: u64 = buf.data.iter().sum();
        assert_eq!(total, 0);
    }

    #[test]
    fn test_local_density_target_merge() {
        let a = LocalDensityTarget::new(10, 10);
        let b = LocalDensityTarget::new(10, 10);
        a.data[0].set(5);
        a.data[1].set(10);
        b.data[0].set(3);
        b.data[1].set(7);
        a.merge(&b);
        assert_eq!(a.data[0].get(), 8);
        assert_eq!(a.data[1].get(), 17);
    }

    #[test]
    fn test_local_density_target_into_density_buffer() {
        let target = LocalDensityTarget::new(10, 10);
        target.data[5].set(42);
        let buf = target.into_density_buffer();
        assert_eq!(buf.data[5], 42);
        assert_eq!(buf.width(), 10);
        assert_eq!(buf.height(), 10);
    }

    // ── AtomicDensityBuffer tests ──────────────────────────────────────

    #[test]
    fn test_atomic_density_buffer_increment() {
        let view = FractalView::new(100, 100);
        let buf = AtomicDensityBuffer::new(100, 100);
        buf.increment(Complex64::new(0.0, 0.0), &view);
        buf.increment(Complex64::new(0.0, 0.0), &view);
        let center_idx = 50 * 100 + 50;
        assert_eq!(buf.data[center_idx].load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_atomic_density_buffer_out_of_bounds() {
        let view = FractalView::new(100, 100);
        let buf = AtomicDensityBuffer::new(100, 100);
        buf.increment(Complex64::new(100.0, 100.0), &view);
        let total: u64 = buf.data.iter().map(|a| a.load(Ordering::Relaxed)).sum();
        assert_eq!(total, 0);
    }

    #[test]
    fn test_atomic_density_buffer_normalize() {
        let buf = AtomicDensityBuffer::new(10, 10);
        buf.data[0].store(0, Ordering::Relaxed);
        buf.data[1].store(50, Ordering::Relaxed);
        buf.data[2].store(100, Ordering::Relaxed);
        buf.data[3].store(1000, Ordering::Relaxed);

        let result = buf.normalize(256, false);
        assert_eq!(result.len(), 100);
        assert_eq!(result[0], 0);
        assert_eq!(result[3], 255);
        for &v in &result {
            assert!(v < 256);
        }
    }

    #[test]
    fn test_atomic_density_buffer_normalize_log() {
        let buf = AtomicDensityBuffer::new(10, 10);
        buf.data[0].store(1, Ordering::Relaxed);
        buf.data[1].store(10, Ordering::Relaxed);
        buf.data[2].store(100, Ordering::Relaxed);
        buf.data[3].store(10000, Ordering::Relaxed);

        let linear = buf.normalize(256, false);
        let log = buf.normalize(256, true);

        // Log should boost low-count pixels relative to linear
        assert!(
            log[0] > linear[0],
            "log normalization should boost low-count pixels: log={} linear={}",
            log[0], linear[0]
        );
    }

    #[test]
    fn test_atomic_density_buffer_normalize_all_zero() {
        let buf = AtomicDensityBuffer::new(10, 10);
        let result = buf.normalize(256, true);
        assert!(result.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_atomic_concurrent_increment() {
        use std::sync::Arc;
        let view = FractalView::new(100, 100);
        let buf = Arc::new(AtomicDensityBuffer::new(100, 100));

        // Spawn threads that all increment the center pixel
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let buf = Arc::clone(&buf);
                let view = view.clone();
                std::thread::spawn(move || {
                    for _ in 0..1000 {
                        buf.increment(Complex64::new(0.0, 0.0), &view);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let center_idx = 50 * 100 + 50;
        assert_eq!(buf.data[center_idx].load(Ordering::Relaxed), 8000);
    }

    // ── Threshold selection test ───────────────────────────────────────

    #[test]
    fn test_threshold_calculation() {
        // Small buffer: 100x100 = 10,000 pixels * 8 = 80,000 bytes — well under 500 MB
        let small = 100usize * 100 * std::mem::size_of::<u64>();
        assert!(small < ATOMIC_THRESHOLD_BYTES, "preview-sized buffer should use fold/reduce path");

        // Large buffer: 15360x8640 = 132,710,400 pixels * 8 = 1,061,683,200 bytes — over threshold
        let large = 15360usize * 8640 * std::mem::size_of::<u64>();
        assert!(large >= ATOMIC_THRESHOLD_BYTES, "supersampled 4K buffer should use atomic path");
    }

    // ── Determinism and thread-independence tests ──────────────────────

    fn make_ifs_params(seed: u64, samples: u64) -> HashMap<String, f64> {
        let mut p = HashMap::new();
        p.insert("num_attractors".to_string(), 2.0);
        p.insert("c0_real".to_string(), -0.5);
        p.insert("c0_imag".to_string(), 0.5);
        p.insert("prob0".to_string(), 0.5);
        p.insert("c1_real".to_string(), -0.5);
        p.insert("c1_imag".to_string(), -0.5);
        p.insert("prob1".to_string(), 0.5);
        p.insert("seed".to_string(), seed as f64);
        p.insert("samples".to_string(), samples as f64);
        p.insert("burn_in".to_string(), 50.0);
        p.insert("use_log_density".to_string(), 1.0);
        p
    }

    #[test]
    fn test_compute_orbit_density_deterministic() {
        use crate::fractals::multi_julia_ifs::MultiJuliaIFS;
        let f = MultiJuliaIFS::new();
        let view = FractalView::new(80, 60);
        let p = make_ifs_params(42, 50_000);

        let r1 = compute_orbit_density(&view, &f, &p, 256, 0);
        let r2 = compute_orbit_density(&view, &f, &p, 256, 0);
        assert_eq!(r1, r2, "Same seed must produce bit-identical output across runs");
    }

    #[test]
    fn test_compute_orbit_density_thread_independent() {
        // Critical test: output must be identical regardless of thread count.
        // The fixed K=64 sub-orbit decomposition ensures this.
        use crate::fractals::multi_julia_ifs::MultiJuliaIFS;
        let f = MultiJuliaIFS::new();
        let view = FractalView::new(80, 60);
        let p = make_ifs_params(42, 50_000);

        let r1 = compute_orbit_density(&view, &f, &p, 256, 1);
        let r8 = compute_orbit_density(&view, &f, &p, 256, 8);
        assert_eq!(r1, r8, "max_threads=1 vs max_threads=8 must produce identical output");
    }

    #[test]
    fn test_compute_orbit_density_valid_range() {
        use crate::fractals::multi_julia_ifs::MultiJuliaIFS;
        let f = MultiJuliaIFS::new();
        let view = FractalView::new(80, 60);
        let p = make_ifs_params(42, 50_000);

        let result = compute_orbit_density(&view, &f, &p, 256, 0);
        assert_eq!(result.len(), 80 * 60);
        for &v in &result {
            assert!(v < 256, "normalized value {} >= max_iter 256", v);
        }
        let nonzero = result.iter().filter(|&&v| v > 0).count();
        assert!(nonzero > 0, "should have some non-zero density pixels");
    }
}
