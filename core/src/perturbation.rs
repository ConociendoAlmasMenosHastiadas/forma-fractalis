//! Perturbation Theory Renderer for deep-zoom fractal rendering.
//!
//! # Overview
//!
//! At extreme zoom levels (zoom > ~1e14), standard f64 arithmetic loses
//! pixel-level precision. Perturbation theory sidesteps this by computing one
//! high-precision "reference orbit" at the view center using `astro_float`
//! BigFloat arithmetic, then approximating every other pixel's orbit as a small
//! f64 delta relative to that reference.
//!
//! ## Mathematics
//!
//! For Mandelbrot (z_{n+1} = z_n^2 + c):
//!
//! Let r_n be the reference orbit at c_ref (view center).
//! For any pixel with dc = c_pixel - c_ref, the delta satisfies:
//!
//!     dz_0 = 0
//!     dz_{n+1} = 2 * r_n * dz_n + dz_n^2 + dc
//!
//! Escape is checked as |dz_n + r_n|^2 > R^2.
//!
//! ## Glitch Handling
//!
//! When |dz_n| > |r_n|, the relative-orbit approximation has broken down.
//! The affected pixel is re-rendered with a direct f64 `iterate()` call
//! (fallback). Glitch counts are reported in `PerturbationResult` and must
//! be surfaced to the user — never hidden.
//!
//! ## Scope
//!
//! This release supports Mandelbrot (power = 2) only. Passing any other fractal
//! to `is_supported_fractal()` returns false, and callers must return a
//! visible error — not a silent fallback.

use astro_float::{BigFloat, RoundingMode};
use rayon::prelude::*;
use crate::fractals::{Fractal, FractalView};
use std::collections::HashMap;

/// Zoom level below which PT is redundant (f64 is sufficient).
///
/// Below this threshold, standard CPU f64 rendering is fully precise. The GUI
/// surfaces this as a warning: "PT active but zoom level does not require it."
pub const PT_LOW_ZOOM_THRESHOLD: f64 = 1e10;

/// Bit width used for BigFloat reference orbit computation.
pub const PT_REFERENCE_BITS: u32 = 128;

/// Maximum number of per-pixel rebase attempts before falling back.
pub const PT_REBASE_BUDGET: usize = 4;

/// When a tiled PT pass still glitches heavily, retry those pixels once with a
/// denser orbit grid before resorting to hi-prec fallback.
const PT_RETRY_TILE_MULTIPLIER: usize = 2;
const PT_RETRY_MAX_TILES: usize = 8;

/// Conservative upper bound on how many initial iterations the truncated
/// series approximation is allowed to skip before the exact delta loop takes
/// over again.
pub const PT_SERIES_MAX_SKIP: usize = 1024;

/// The combined higher-order correction terms must stay comfortably below the
/// full orbit value before the warm start is allowed to skip to that step.
const PT_SERIES_CORRECTION_FACTOR: f64 = 0.0625;

/// The warm start only skips through iterates that are still well inside the
/// escape radius. This avoids jumping across early exterior escapes.
const PT_SERIES_ESCAPE_MARGIN_FACTOR: f64 = 0.5;

#[derive(Clone, Copy, Default)]
struct PtSeriesStats {
    accepted: bool,
    skipped_iterations: usize,
    rejected_escape_margin: bool,
    rejected_correction: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Data types
// ─────────────────────────────────────────────────────────────────────────────

/// A high-precision reference orbit computed at the view center.
///
/// The orbit contains `r_n = (Re(z_n), Im(z_n))` in f64 for each iteration
/// step n from 0 up to (but not including) the escape iteration (or `max_iter`
/// if the reference point did not escape).
///
/// Fields are public so the GUI layer can inspect them (e.g., to invalidate
/// the cache when view parameters change).
pub struct ReferenceOrbit {
    /// Per-iteration (real, imag) pairs truncated from BigFloat to f64.
    ///
    /// - When `escaped == true`:  entries are z_0 .. z_k where z_k is the first
    ///   iterate whose magnitude squared exceeds the escape radius.  Length = k+1.
    /// - When `escaped == false`: entries are z_0 .. z_{max_iter} (one extra step
    ///   beyond the loop limit so `pt_iterate` can match the last step that
    ///   `Mandelbrot::iterate` checks).  Length = max_iter + 1.
    pub orbit: Vec<(f64, f64)>,

    /// True when the reference orbit escaped within `max_iter` steps.
    /// False when the reference point is in the Mandelbrot set at this depth.
    /// Use this flag — not `orbit.len() < max_iter` — to detect early escape.
    pub escaped: bool,

    /// Center coordinates used to compute this orbit.
    pub center_x: f64,
    pub center_y: f64,

    /// Zoom level used to compute this orbit.
    pub zoom: f64,

    /// Maximum iteration limit used.
    pub max_iter: u32,

    /// BigFloat precision (bits) used to compute this orbit.
    /// Changing precision invalidates the cache.
    pub bits: u32,

    /// Position of the reference point within the view, as a fraction [0, 1].
    /// Used to compute dc = (pixel_frac - tile_frac) * range.
    /// (0.5, 0.5) means the view center (default for single-orbit mode).
    /// For a tile grid, each orbit stores the center of its tile.
    pub tile_frac_x: f64,
    pub tile_frac_y: f64,

    /// First-order coefficient prefix for a conservative Mandelbrot series
    /// warm start. Only the first `PT_SERIES_MAX_SKIP` steps are stored.
    series_linear: Vec<(f64, f64)>,

    /// Second-order coefficient prefix for the same warm start.
    series_quadratic: Vec<(f64, f64)>,

    /// Third-order coefficient prefix for the same warm start.
    series_cubic: Vec<(f64, f64)>,
}

/// Output from `render_perturbation()`.
pub struct PerturbationResult {
    /// Iteration counts for every pixel in row-major order.
    pub iterations: Vec<u32>,

    /// Number of pixels that still required the direct fallback path after
    /// any rebase attempts were exhausted or failed.
    /// Should be surfaced in the UI — never suppressed.
    pub glitch_count: usize,

    /// Per-pixel glitch flag, parallel to `iterations`.
    /// `true` means the pixel was re-rendered via the direct fallback path;
    /// `false` means the delta recurrence succeeded, possibly after rebasing.
    /// Use this to compare only delta-path pixels against a f64 baseline.
    pub glitch_mask: Vec<bool>,

    /// Total number of successful rebase events across all pixels.
    pub rebase_count: usize,

    /// Number of pixels that successfully rebased at least once.
    pub rebased_pixel_count: usize,

    /// Number of pixels that fell back only after consuming the full rebase budget.
    pub rebase_exhausted_count: usize,

    /// Number of pixels where the SA warm start was accepted.
    pub sa_accepted_pixel_count: usize,

    /// Total iterations skipped by accepted SA warm starts.
    pub sa_total_skipped_iterations: usize,

    /// Largest accepted SA skip on any pixel in this render.
    pub sa_max_skipped_iterations: usize,

    /// Pixels where the SA candidate was stopped by the escape-margin guard.
    pub sa_rejected_escape_margin_count: usize,

    /// Pixels where the SA candidate was stopped by the higher-order correction guard.
    pub sa_rejected_correction_count: usize,

    /// True when `view.zoom < PT_LOW_ZOOM_THRESHOLD`.
    /// PT is unnecessary at this zoom level; suggest switching to CPU mode.
    pub low_zoom_warning: bool,
}

#[derive(Clone, Copy)]
struct RebasePoint {
    iteration: usize,
    total_re: f64,
    total_im: f64,
}

enum PtIterOutcome {
    Finished(u32),
    NeedRebase(RebasePoint),
}

#[derive(Clone, Copy)]
enum PtPixelOutcome {
    Finished {
        iterations: u32,
        rebase_count: usize,
        series_stats: PtSeriesStats,
    },
    NeedsFallback {
        rebase_count: usize,
        budget_exhausted: bool,
        series_stats: PtSeriesStats,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// Reference orbit
/// Convert a `BigFloat` value to f64 for orbit storage.
///
/// astro-float 0.9 has no `to_f64()` method; we use the Display representation
/// (standard scientific notation) as an intermediate, which round-trips cleanly
/// for values within the f64 range. NaN/Inf are mapped to their f64 equivalents.
fn bf_to_f64(bf: &BigFloat) -> f64 {
    if bf.is_nan() {
        return f64::NAN;
    }
    if bf.is_inf() {
        return if bf.is_inf_pos() { f64::INFINITY } else { f64::NEG_INFINITY };
    }
    format!("{}", bf).parse::<f64>().unwrap_or(0.0)
}

fn complex_mul(a_re: f64, a_im: f64, b_re: f64, b_im: f64) -> (f64, f64) {
    (
        a_re * b_re - a_im * b_im,
        a_re * b_im + a_im * b_re,
    )
}

/// Shared BigFloat orbit computation kernel.
///
/// Returns `(orbit, escaped)` where:
/// - `escaped = true`:  the orbit broke early; last entry is the escaping z_k.
/// - `escaped = false`: the orbit ran all `max_iter` steps without escape and
///   one additional entry z_{max_iter} is appended.  This extra entry lets
///   `pt_iterate` check the same final step that `Mandelbrot::iterate` checks
///   (direct iteration checks z_{max_iter} at its last step and can return
///   `max_iter - 1`; without the extra entry PT could only return `max_iter`).
fn compute_orbit_bf(cr: &BigFloat, ci: &BigFloat, max_iter: u32, bits: u32) -> (Vec<(f64, f64)>, bool) {
    let p = bits as usize;
    let rm = RoundingMode::ToEven;
    let four = BigFloat::from_f64(4.0, p);
    let two  = BigFloat::from_f64(2.0, p);
    let mut zr = BigFloat::from_f64(0.0, p);
    let mut zi = BigFloat::from_f64(0.0, p);
    // Capacity: max_iter + 1 to avoid reallocation for the extra step.
    let mut orbit = Vec::with_capacity(max_iter as usize + 1);
    let mut escaped = false;
    for _ in 0..max_iter {
        let zr2 = zr.mul(&zr, p, rm);
        let zi2 = zi.mul(&zi, p, rm);
        let norm_sq = zr2.add(&zi2, p, rm);
        orbit.push((bf_to_f64(&zr), bf_to_f64(&zi)));
        if norm_sq.cmp(&four).map_or(false, |v| v > 0) {
            escaped = true;
            break;
        }
        let new_zr = zr2.sub(&zi2, p, rm).add(cr, p, rm);
        let new_zi = two.mul(&zr, p, rm).mul(&zi, p, rm).add(ci, p, rm);
        if new_zr.is_nan() || new_zr.is_inf() || new_zi.is_nan() || new_zi.is_inf() {
            escaped = true;
            break;
        }
        zr = new_zr;
        zi = new_zi;
    }
    if !escaped {
        // After the loop, zr/zi hold z_{max_iter} (computed in the last update
        // but not yet pushed).  Pushing it lets pt_iterate cover this step,
        // matching Mandelbrot::iterate's final escape check.
        orbit.push((bf_to_f64(&zr), bf_to_f64(&zi)));
    }
    (orbit, escaped)
}

fn compute_series_coefficients_prefix(
    orbit: &[(f64, f64)],
) -> (Vec<(f64, f64)>, Vec<(f64, f64)>, Vec<(f64, f64)>) {
    let prefix_len = orbit.len().min(PT_SERIES_MAX_SKIP + 1);
    let mut linear = Vec::with_capacity(prefix_len);
    let mut quadratic = Vec::with_capacity(prefix_len);
    let mut cubic = Vec::with_capacity(prefix_len);

    if prefix_len == 0 {
        return (linear, quadratic, cubic);
    }

    linear.push((0.0, 0.0));
    quadratic.push((0.0, 0.0));
    cubic.push((0.0, 0.0));

    for n in 0..prefix_len.saturating_sub(1) {
        let (r_re, r_im) = orbit[n];
        let (a_re, a_im) = linear[n];
        let (b_re, b_im) = quadratic[n];
        let (c_re, c_im) = cubic[n];
        let (two_r_a_re, two_r_a_im) = complex_mul(2.0 * r_re, 2.0 * r_im, a_re, a_im);
        let (two_r_b_re, two_r_b_im) = complex_mul(2.0 * r_re, 2.0 * r_im, b_re, b_im);
        let (two_r_c_re, two_r_c_im) = complex_mul(2.0 * r_re, 2.0 * r_im, c_re, c_im);
        let (a_sq_re, a_sq_im) = complex_mul(a_re, a_im, a_re, a_im);
        let (two_a_b_re, two_a_b_im) = complex_mul(2.0 * a_re, 2.0 * a_im, b_re, b_im);

        linear.push((two_r_a_re + 1.0, two_r_a_im));
        quadratic.push((two_r_b_re + a_sq_re, two_r_b_im + a_sq_im));
        cubic.push((two_r_c_re + two_a_b_re, two_r_c_im + two_a_b_im));
    }

    (linear, quadratic, cubic)
}

fn try_series_skip(
    orbit: &ReferenceOrbit,
    dc_re: f64,
    dc_im: f64,
    escape_radius_sq: f64,
) -> (Option<(usize, f64, f64)>, PtSeriesStats) {
    if orbit.series_linear.len() <= 1 || (dc_re == 0.0 && dc_im == 0.0) {
        return (None, PtSeriesStats::default());
    }

    let safe_escape_sq = escape_radius_sq * PT_SERIES_ESCAPE_MARGIN_FACTOR;
    let (dc_sq_re, dc_sq_im) = complex_mul(dc_re, dc_im, dc_re, dc_im);
    let (dc_cu_re, dc_cu_im) = complex_mul(dc_sq_re, dc_sq_im, dc_re, dc_im);
    let mut best = None;
    let mut stats = PtSeriesStats::default();

    for n in 1..orbit.series_linear.len() {
        let (a_re, a_im) = orbit.series_linear[n];
        let (b_re, b_im) = orbit.series_quadratic[n];
        let (c_re, c_im) = orbit.series_cubic[n];
        let (linear_re, linear_im) = complex_mul(a_re, a_im, dc_re, dc_im);
        let (quadratic_re, quadratic_im) = complex_mul(b_re, b_im, dc_sq_re, dc_sq_im);
        let (cubic_re, cubic_im) = complex_mul(c_re, c_im, dc_cu_re, dc_cu_im);
        let dz_re = linear_re + quadratic_re + cubic_re;
        let dz_im = linear_im + quadratic_im + cubic_im;
        let (r_re, r_im) = orbit.orbit[n];
        let total_re = r_re + dz_re;
        let total_im = r_im + dz_im;

        if !dz_re.is_finite() || !dz_im.is_finite() || !total_re.is_finite() || !total_im.is_finite() {
            break;
        }

        let total_norm_sq = total_re * total_re + total_im * total_im;
        let quadratic_norm_sq = quadratic_re * quadratic_re + quadratic_im * quadratic_im;
        let cubic_norm_sq = cubic_re * cubic_re + cubic_im * cubic_im;

        if total_norm_sq >= safe_escape_sq {
            stats.rejected_escape_margin = true;
            break;
        }

        if quadratic_norm_sq + cubic_norm_sq > PT_SERIES_CORRECTION_FACTOR * total_norm_sq {
            stats.rejected_correction = true;
            break;
        }

        best = Some((n, dz_re, dz_im));
    }

    if let Some((skip_iteration, _, _)) = best {
        stats.accepted = true;
        stats.skipped_iterations = skip_iteration;
    }

    (best, stats)
}

// ─────────────────────────────────────────────────────────────────────────────

impl ReferenceOrbit {
    /// Compute a Mandelbrot reference orbit at (`center_x`, `center_y`) using
    /// BigFloat arithmetic at `bits` precision.
    ///
    /// The reference orbit is z_{n+1} = z_n^2 + c where z_0 = 0 and c is the
    /// view center. The orbit stores z_n (before squaring) so that the caller's
    /// delta loop can use `r_n` in the recurrence.
    ///
    /// `bits` should be `PT_REFERENCE_BITS` (128) or higher for deep zooms.
    pub fn compute_mandelbrot(
        center_x: f64,
        center_y: f64,
        zoom: f64,
        max_iter: u32,
        bits: u32,
    ) -> Self {
        let p = bits as usize;
        let cr = BigFloat::from_f64(center_x, p);
        let ci = BigFloat::from_f64(center_y, p);
        let (orbit, escaped) = compute_orbit_bf(&cr, &ci, max_iter, bits);
        let (series_linear, series_quadratic, series_cubic) = compute_series_coefficients_prefix(&orbit);
        Self {
            orbit,
            escaped,
            center_x,
            center_y,
            zoom,
            max_iter,
            bits,
            tile_frac_x: 0.5,
            tile_frac_y: 0.5,
            series_linear,
            series_quadratic,
            series_cubic,
        }
    }

    /// Compute a reference orbit whose center is a specific pixel, using
    /// `screen_to_complex_hiprec` so that even at extreme zoom the center
    /// coordinates are BigFloat-accurate from the start.
    ///
    /// `tile_frac_x/y` are stored on the orbit so that the render loop can
    /// compute `dc = (pixel_frac - tile_frac) * range` correctly.
    pub fn compute_at_pixel(
        view: &FractalView,
        tile_px: u32,
        tile_py: u32,
        max_iter: u32,
        bits: u32,
    ) -> Self {
        let (cr, ci) = view.screen_to_complex_hiprec(tile_px, tile_py, bits);
        let (orbit, escaped) = compute_orbit_bf(&cr, &ci, max_iter, bits);
        let (series_linear, series_quadratic, series_cubic) = compute_series_coefficients_prefix(&orbit);
        // tile_frac_x/y must match the screen_to_complex convention: pixel x maps to
        // (x - width/2) * range / width, so the orbit at pixel tile_px corresponds to
        // fractional position tile_px / width (NOT (tile_px + 0.5) / width).  Using +0.5
        // would introduce a constant 0.5-pixel offset in every dc computation.
        let tile_frac_x = tile_px as f64 / view.width as f64;
        let tile_frac_y = tile_py as f64 / view.height as f64;
        Self {
            orbit,
            escaped,
            center_x: bf_to_f64(&cr),
            center_y: bf_to_f64(&ci),
            zoom: view.zoom,
            max_iter,
            bits,
            tile_frac_x,
            tile_frac_y,
            series_linear,
            series_quadratic,
            series_cubic,
        }
    }

    /// Compute a grid of `tiles x tiles` reference orbits covering the view.
    ///
    /// Each orbit is centered on its tile in complex space, computed with
    /// BigFloat precision via `compute_at_pixel`. A 2x2 grid produces 4 orbits,
    /// 4x4 produces 16, etc.
    ///
    /// With multiple orbits each pixel's `dc` is relative to a closer reference,
    /// so the perturbation approximation holds for a larger fraction of pixels
    /// — directly reducing the expensive hi-prec fallback rate.
    pub fn compute_tile_orbits(
        view: &FractalView,
        tiles: u32,
        max_iter: u32,
        bits: u32,
    ) -> Vec<Self> {
        let t = tiles as usize;
        let w = view.width as usize;
        let h = view.height as usize;
        let mut orbits = Vec::with_capacity(t * t);
        for ty in 0..t {
            for tx in 0..t {
                // Center pixel of tile (tx, ty): w*(2*tx+1) / (2*t)
                let tile_px = (w * (2 * tx + 1) / (2 * t)) as u32;
                let tile_py = (h * (2 * ty + 1) / (2 * t)) as u32;
                orbits.push(Self::compute_at_pixel(view, tile_px, tile_py, max_iter, bits));
            }
        }
        orbits
    }

    /// Returns true when this cached orbit can be reused for the given parameters.
    pub fn is_valid_for(&self, center_x: f64, center_y: f64, zoom: f64, max_iter: u32, bits: u32) -> bool {
        // Bit-exact comparison of all parameters. Any change requires recomputation.
        self.center_x == center_x
            && self.center_y == center_y
            && self.zoom == zoom
            && self.max_iter == max_iter
            && self.bits == bits
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Fractal support check
// ─────────────────────────────────────────────────────────────────────────────

/// Returns true when perturbation theory rendering is valid for the given
/// fractal and parameters.
///
/// Currently only Mandelbrot with power = 2 is supported. Callers that receive
/// false MUST surface an error message — silent fallback is forbidden.
pub fn is_supported_fractal(fractal: &dyn Fractal, params: &HashMap<String, f64>) -> bool {
    if fractal.name() != "Mandelbrot" {
        return false;
    }
    let power = params.get("power").copied().unwrap_or(2.0);
    (power - 2.0).abs() < f64::EPSILON
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-pixel delta iteration
// ─────────────────────────────────────────────────────────────────────────────

/// Compute iteration count for one pixel using the perturbation-theory delta
/// recurrence.
///
/// Returns `Finished(n)` when the pixel escaped at iteration `n`, or
/// `Finished(max_iter)` when the reference did not escape and neither did the delta.
///
/// Returns `NeedRebase` when the current reference stops being representative.
/// The caller can then select a different reference orbit and resume from the
/// returned `(iteration, z_total)` state instead of immediately falling back.
fn pt_iterate_from(
    orbit: &ReferenceOrbit,
    start_iteration: usize,
    initial_dz_re: f64,
    initial_dz_im: f64,
    dc_re: f64,
    dc_im: f64,
    escape_radius_sq: f64,
    glitch_tolerance: f64,
) -> PtIterOutcome {
    let mut dz_re = initial_dz_re;
    let mut dz_im = initial_dz_im;

    for n in start_iteration..orbit.orbit.len() {
        let (r_re, r_im) = orbit.orbit[n];
        // Total z = dz + r  (the actual iterate value for this pixel)
        let total_re = dz_re + r_re;
        let total_im = dz_im + r_im;

        // Escape check on the full orbit value.
        //
        // n = 0 never escapes: r_0 = (0,0), dz_0 = (0,0), total = 0.
        // The first possible escape is at n = 1, where total = c_pixel.
        //
        // `Mandelbrot::iterate()` starts with z = c and returns iter = 0 for the
        // same condition, so we subtract 1 to keep the counts consistent.
        // n.saturating_sub(1) is used for safety, but n = 0 is mathematically
        // unreachable here.
        if total_re * total_re + total_im * total_im > escape_radius_sq {
            return PtIterOutcome::Finished(n.saturating_sub(1) as u32);
        }

        // Glitch check: the perturbation approximation breaks down when the
        // delta |dz| has grown larger than the total orbit value |z_total|
        // (= |r + dz|).  Using |r| alone as the denominator causes massive
        // false-positive glitches whenever the reference orbit passes near
        // zero — a common event for Mandelbrot boundary orbits.  The total-
        // orbit check is the correct Pauldelbrot/Zhuoran criterion.
        //
        // `glitch_tolerance` relaxes this: a value of 1.0 is mathematically
        // strict (glitch when |dz| >= |z_total|). Higher values (e.g. 4.0)
        // allow the delta to be proportionally larger before triggering the
        // fallback, trading exactness for fewer expensive hi-prec fallbacks.
        let dz_norm_sq = dz_re * dz_re + dz_im * dz_im;
        let total_norm_sq = total_re * total_re + total_im * total_im;
        if dz_norm_sq > glitch_tolerance * total_norm_sq {
            return PtIterOutcome::NeedRebase(RebasePoint {
                iteration: n,
                total_re,
                total_im,
            });
        }

        // If the current reference orbit has no next step but this pixel still
        // has not escaped, rebasing is the only chance to stay on the PT path.
        if n + 1 >= orbit.orbit.len() {
            return if orbit.escaped {
                PtIterOutcome::NeedRebase(RebasePoint {
                    iteration: n,
                    total_re,
                    total_im,
                })
            } else {
                PtIterOutcome::Finished(orbit.max_iter)
            };
        }

        // Delta recurrence: dz_{n+1} = 2*r_n*dz + dz^2 + dc
        let two_r_dz_re = 2.0 * (r_re * dz_re - r_im * dz_im);
        let two_r_dz_im = 2.0 * (r_re * dz_im + r_im * dz_re);
        let dz_sq_re = dz_re * dz_re - dz_im * dz_im;
        let dz_sq_im = 2.0 * dz_re * dz_im;

        dz_re = two_r_dz_re + dz_sq_re + dc_re;
        dz_im = two_r_dz_im + dz_sq_im + dc_im;
    }

    PtIterOutcome::Finished(orbit.max_iter)
}

fn orbit_dc(
    px_frac: f64,
    py_frac: f64,
    range_x: f64,
    range_y: f64,
    orbit: &ReferenceOrbit,
) -> (f64, f64) {
    (
        (px_frac - orbit.tile_frac_x) * range_x,
        (py_frac - orbit.tile_frac_y) * range_y,
    )
}

fn tile_orbit_index(
    px: usize,
    py: usize,
    width: usize,
    height: usize,
    tiles_w: usize,
    tiles_h: usize,
) -> usize {
    let tx = (px * tiles_w / width).min(tiles_w - 1);
    let ty = (py * tiles_h / height).min(tiles_h - 1);
    ty * tiles_w + tx
}

fn merge_series_stats(primary: PtSeriesStats, retry: PtSeriesStats) -> PtSeriesStats {
    PtSeriesStats {
        accepted: primary.accepted || retry.accepted,
        skipped_iterations: primary.skipped_iterations.max(retry.skipped_iterations),
        rejected_escape_margin: primary.rejected_escape_margin || retry.rejected_escape_margin,
        rejected_correction: primary.rejected_correction || retry.rejected_correction,
    }
}

fn fallback_iteration_for_pixel(
    view: &FractalView,
    px: usize,
    py: usize,
    range_x: f64,
    range_y: f64,
    fractal: &dyn Fractal,
    fractal_params: &HashMap<String, f64>,
    max_iter: u32,
    hiprec_bits: u32,
) -> u32 {
    let px_frac = px as f64 / view.width as f64;
    let py_frac = py as f64 / view.height as f64;

    if fractal.supports_hiprec() && hiprec_bits > 64 {
        let (c_re_bf, c_im_bf) = view.screen_to_complex_hiprec(px as u32, py as u32, hiprec_bits);
        fractal.iterate_hiprec(&c_re_bf, &c_im_bf, fractal_params, max_iter, hiprec_bits)
    } else {
        let c_re = view.center_x + (px_frac - 0.5) * range_x;
        let c_im = view.center_y + (py_frac - 0.5) * range_y;
        fractal.iterate(c_re, c_im, fractal_params, max_iter)
    }
}

fn pt_outcome_for_pixel(
    px: usize,
    py: usize,
    width: usize,
    height: usize,
    range_x: f64,
    range_y: f64,
    orbits: &[ReferenceOrbit],
    tiles_w: usize,
    tiles_h: usize,
    escape_radius_sq: f64,
    glitch_tolerance: f64,
) -> PtPixelOutcome {
    let px_frac = px as f64 / width as f64;
    let py_frac = py as f64 / height as f64;
    pt_iterate_rebased(
        px_frac,
        py_frac,
        range_x,
        range_y,
        orbits,
        tile_orbit_index(px, py, width, height, tiles_w, tiles_h),
        escape_radius_sq,
        glitch_tolerance,
    )
}

fn choose_rebase_orbit(
    orbits: &[ReferenceOrbit],
    current_orbit_index: usize,
    iteration: usize,
    total_re: f64,
    total_im: f64,
    glitch_tolerance: f64,
) -> Option<(usize, f64, f64)> {
    let total_norm_sq = total_re * total_re + total_im * total_im;
    let mut best: Option<(usize, f64, f64, f64)> = None;

    for (orbit_index, orbit) in orbits.iter().enumerate() {
        if orbit_index == current_orbit_index {
            continue;
        }

        let Some(&(candidate_re, candidate_im)) = orbit.orbit.get(iteration) else {
            continue;
        };

        let dz_re = total_re - candidate_re;
        let dz_im = total_im - candidate_im;
        let dz_norm_sq = dz_re * dz_re + dz_im * dz_im;

        if dz_norm_sq > glitch_tolerance * total_norm_sq {
            continue;
        }

        if best.map_or(true, |(_, _, _, best_norm_sq)| dz_norm_sq < best_norm_sq) {
            best = Some((orbit_index, dz_re, dz_im, dz_norm_sq));
        }
    }

    best.map(|(orbit_index, dz_re, dz_im, _)| (orbit_index, dz_re, dz_im))
}

fn pt_iterate_rebased(
    px_frac: f64,
    py_frac: f64,
    range_x: f64,
    range_y: f64,
    orbits: &[ReferenceOrbit],
    initial_orbit_index: usize,
    escape_radius_sq: f64,
    glitch_tolerance: f64,
) -> PtPixelOutcome {
    let mut orbit_index = initial_orbit_index;
    let mut start_iteration = 0usize;
    let mut dz_re = 0.0_f64;
    let mut dz_im = 0.0_f64;
    let mut dc = orbit_dc(px_frac, py_frac, range_x, range_y, &orbits[orbit_index]);
    let mut rebase_count = 0usize;
    let mut remaining_budget = PT_REBASE_BUDGET;
    let (series_skip, series_stats) = try_series_skip(&orbits[orbit_index], dc.0, dc.1, escape_radius_sq);

    if let Some((skip_iteration, skip_dz_re, skip_dz_im)) = series_skip {
        start_iteration = skip_iteration;
        dz_re = skip_dz_re;
        dz_im = skip_dz_im;
    }

    loop {
        match pt_iterate_from(
            &orbits[orbit_index],
            start_iteration,
            dz_re,
            dz_im,
            dc.0,
            dc.1,
            escape_radius_sq,
            glitch_tolerance,
        ) {
            PtIterOutcome::Finished(iterations) => {
                return PtPixelOutcome::Finished {
                    iterations,
                    rebase_count,
                    series_stats,
                };
            }
            PtIterOutcome::NeedRebase(point) => {
                if remaining_budget == 0 {
                    return PtPixelOutcome::NeedsFallback {
                        rebase_count,
                        budget_exhausted: true,
                        series_stats,
                    };
                }

                let Some((next_orbit_index, next_dz_re, next_dz_im)) = choose_rebase_orbit(
                    orbits,
                    orbit_index,
                    point.iteration,
                    point.total_re,
                    point.total_im,
                    glitch_tolerance,
                ) else {
                    return PtPixelOutcome::NeedsFallback {
                        rebase_count,
                        budget_exhausted: false,
                        series_stats,
                    };
                };

                orbit_index = next_orbit_index;
                start_iteration = point.iteration;
                dz_re = next_dz_re;
                dz_im = next_dz_im;
                dc = orbit_dc(px_frac, py_frac, range_x, range_y, &orbits[orbit_index]);
                rebase_count += 1;
                remaining_budget -= 1;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Full-frame render
// ─────────────────────────────────────────────────────────────────────────────

/// Render an entire frame using perturbation theory.
///
/// Each pixel's complex offset `dc` from the view center is computed in f64
/// (sufficient for zoom levels where the pixel delta is representable). The
/// delta recurrence runs against `orbit`. Glitched pixels are re-rendered
/// via the fractal's `iterate_hiprec()` method at `hiprec_bits` precision,
/// which avoids the f64 coordinate-precision problem that causes tearing at
/// deep zoom.
///
/// The returned `PerturbationResult` includes iteration counts, a glitch count,
/// and a flag indicating whether the zoom level actually required PT.
///
/// # Precondition
///
/// `orbit` must be a Mandelbrot orbit computed at the same `view.center_x` /
/// `view.center_y` / `view.zoom` as the view passed here. Call
/// `is_supported_fractal()` before this function and return an error if false.
/// `glitch_tolerance` is a multiplier on the glitch threshold:
/// `|dz|² > glitch_tolerance * |z_total|²`. Use 1.0 for mathematically strict
/// behavior. Higher values (e.g. 4.0) allow a larger delta before triggering the
/// hi-prec fallback — fewer glitches, slightly less exact coloring.
pub fn render_perturbation(
    view: &FractalView,
    orbit: &ReferenceOrbit,
    fractal: &dyn Fractal,
    fractal_params: &HashMap<String, f64>,
    max_iter: u32,
    hiprec_bits: u32,
    max_threads: usize,
    glitch_tolerance: f64,
) -> PerturbationResult {
    render_perturbation_tiled(
        view,
        std::slice::from_ref(orbit),
        1, 1,
        fractal, fractal_params, max_iter, hiprec_bits, max_threads, glitch_tolerance,
    )
}

/// Render using a grid of `tiles_w × tiles_h` reference orbits.
///
/// Each pixel is assigned to the tile it falls in and uses that tile's orbit
/// as its reference. The delta `dc` is computed relative to the tile center
/// rather than the view center, so `|dc|` is at most `range / (2 * tiles)`
/// instead of `range / 2` — dramatically reducing the glitch rate when the
/// view center happens to be outside the Mandelbrot set.
///
/// For `tiles_w == tiles_h == 1` with an orbit from `ReferenceOrbit::compute_mandelbrot`,
/// this is identical to the old single-orbit behavior.
///
/// # No stitching artifacts
/// Each pixel independently computes an exact delta to its tile's reference.
/// Nothing is blended or interpolated at tile boundaries — they are purely a
/// bookkeeping division.
pub fn render_perturbation_tiled(
    view: &FractalView,
    orbits: &[ReferenceOrbit],
    tiles_w: usize,
    tiles_h: usize,
    fractal: &dyn Fractal,
    fractal_params: &HashMap<String, f64>,
    max_iter: u32,
    hiprec_bits: u32,
    max_threads: usize,
    glitch_tolerance: f64,
) -> PerturbationResult {
    assert_eq!(orbits.len(), tiles_w * tiles_h,
        "orbit slice length ({}) must equal tiles_w * tiles_h ({})",
        orbits.len(), tiles_w * tiles_h);

    let escape_r = fractal_params.get("escape_radius").copied().unwrap_or(2.0);
    let escape_radius_sq = escape_r * escape_r;

    let scale = 3.5 / view.zoom;
    let aspect = view.width as f64 / view.height as f64;
    let range_x = scale * aspect;
    let range_y = scale;

    let width = view.width as usize;
    let height = view.height as usize;
    let pixel_count = width * height;

    let low_zoom_warning = view.zoom < PT_LOW_ZOOM_THRESHOLD;

    let compute_primary = || -> Vec<PtPixelOutcome> {
        (0..pixel_count)
        .into_par_iter()
        .map(|idx| {
            let px = idx % width;
            let py = idx / width;

            pt_outcome_for_pixel(
                px,
                py,
                width,
                height,
                range_x,
                range_y,
                orbits,
                tiles_w,
                tiles_h,
                escape_radius_sq,
                glitch_tolerance,
            )
        })
        .collect()
    };

    let global_count = rayon::current_num_threads();
    let primary_outcomes: Vec<PtPixelOutcome> = if max_threads > 0 && max_threads < global_count {
        match rayon::ThreadPoolBuilder::new().num_threads(max_threads).build() {
            Ok(pool) => pool.install(|| compute_primary()),
            Err(_) => compute_primary(),
        }
    } else {
        compute_primary()
    };

    let retry_tiles = if tiles_w == tiles_h && tiles_w < PT_RETRY_MAX_TILES {
        Some((tiles_w * PT_RETRY_TILE_MULTIPLIER).min(PT_RETRY_MAX_TILES))
    } else {
        None
    };
    let fallback_candidates = primary_outcomes
        .iter()
        .filter(|outcome| matches!(outcome, PtPixelOutcome::NeedsFallback { .. }))
        .count();
    let retry_orbits = if fallback_candidates > 0 {
        retry_tiles.map(|tiles| ReferenceOrbit::compute_tile_orbits(view, tiles as u32, max_iter, hiprec_bits))
    } else {
        None
    };

    let finalize = || -> Vec<(u32, bool, usize, bool, PtSeriesStats)> {
        (0..pixel_count)
            .into_par_iter()
            .map(|idx| {
                let px = idx % width;
                let py = idx / width;

                match primary_outcomes[idx] {
                    PtPixelOutcome::Finished {
                        iterations,
                        rebase_count,
                        series_stats,
                    } => (iterations, false, rebase_count, false, series_stats),
                    PtPixelOutcome::NeedsFallback {
                        rebase_count,
                        budget_exhausted,
                        series_stats,
                    } => {
                        if let (Some(orbits), Some(tiles)) = (retry_orbits.as_ref(), retry_tiles) {
                            match pt_outcome_for_pixel(
                                px,
                                py,
                                width,
                                height,
                                range_x,
                                range_y,
                                orbits,
                                tiles,
                                tiles,
                                escape_radius_sq,
                                glitch_tolerance,
                            ) {
                                PtPixelOutcome::Finished {
                                    iterations,
                                    rebase_count: retry_rebases,
                                    series_stats: retry_stats,
                                } => {
                                    return (
                                        iterations,
                                        false,
                                        rebase_count + retry_rebases,
                                        false,
                                        merge_series_stats(series_stats, retry_stats),
                                    );
                                }
                                PtPixelOutcome::NeedsFallback {
                                    rebase_count: retry_rebases,
                                    budget_exhausted: retry_budget_exhausted,
                                    series_stats: retry_stats,
                                } => {
                                    let n = fallback_iteration_for_pixel(
                                        view,
                                        px,
                                        py,
                                        range_x,
                                        range_y,
                                        fractal,
                                        fractal_params,
                                        max_iter,
                                        hiprec_bits,
                                    );
                                    return (
                                        n,
                                        true,
                                        rebase_count + retry_rebases,
                                        budget_exhausted || retry_budget_exhausted,
                                        merge_series_stats(series_stats, retry_stats),
                                    );
                                }
                            }
                        }

                        let n = fallback_iteration_for_pixel(
                            view,
                            px,
                            py,
                            range_x,
                            range_y,
                            fractal,
                            fractal_params,
                            max_iter,
                            hiprec_bits,
                        );
                        (n, true, rebase_count, budget_exhausted, series_stats)
                    }
                }
            })
            .collect()
    };

    let results: Vec<(u32, bool, usize, bool, PtSeriesStats)> = if max_threads > 0 && max_threads < global_count {
        match rayon::ThreadPoolBuilder::new().num_threads(max_threads).build() {
            Ok(pool) => pool.install(|| finalize()),
            Err(_) => finalize(),
        }
    } else {
        finalize()
    };

    let glitch_count = results.iter().filter(|(_, used_fallback, _, _, _)| *used_fallback).count();
    let rebase_count = results.iter().map(|(_, _, pixel_rebases, _, _)| *pixel_rebases).sum();
    let rebased_pixel_count = results.iter().filter(|(_, _, pixel_rebases, _, _)| *pixel_rebases > 0).count();
    let rebase_exhausted_count = results
        .iter()
        .filter(|(_, used_fallback, _, budget_exhausted, _)| *used_fallback && *budget_exhausted)
        .count();
    let sa_accepted_pixel_count = results
        .iter()
        .filter(|(_, _, _, _, series_stats)| series_stats.accepted)
        .count();
    let sa_total_skipped_iterations = results
        .iter()
        .map(|(_, _, _, _, series_stats)| series_stats.skipped_iterations)
        .sum();
    let sa_max_skipped_iterations = results
        .iter()
        .map(|(_, _, _, _, series_stats)| series_stats.skipped_iterations)
        .max()
        .unwrap_or(0);
    let sa_rejected_escape_margin_count = results
        .iter()
        .filter(|(_, _, _, _, series_stats)| series_stats.rejected_escape_margin)
        .count();
    let sa_rejected_correction_count = results
        .iter()
        .filter(|(_, _, _, _, series_stats)| series_stats.rejected_correction)
        .count();
    let glitch_mask: Vec<bool> = results.iter().map(|(_, used_fallback, _, _, _)| *used_fallback).collect();
    let iterations = results.into_iter().map(|(n, _, _, _, _)| n).collect();

    PerturbationResult {
        iterations,
        glitch_count,
        glitch_mask,
        rebase_count,
        rebased_pixel_count,
        rebase_exhausted_count,
        sa_accepted_pixel_count,
        sa_total_skipped_iterations,
        sa_max_skipped_iterations,
        sa_rejected_escape_margin_count,
        sa_rejected_correction_count,
        low_zoom_warning,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fractals::Mandelbrot;

    fn mandelbrot_params(power: f64) -> HashMap<String, f64> {
        let mut p = HashMap::new();
        p.insert("power".to_string(), power);
        p.insert("escape_radius".to_string(), 2.0);
        p
    }

    fn scene_view(center_x: f64, center_y: f64, zoom: f64, width: u32, height: u32) -> crate::fractals::FractalView {
        let mut view = crate::fractals::FractalView::new(width, height);
        view.center_x = center_x;
        view.center_y = center_y;
        view.zoom = zoom;
        view
    }

    // ── is_supported_fractal ──────────────────────────────────────────────

    #[test]
    fn supported_mandelbrot_power2() {
        let m = Mandelbrot::new();
        let params = mandelbrot_params(2.0);
        assert!(is_supported_fractal(&m, &params));
    }

    #[test]
    fn unsupported_mandelbrot_power3() {
        let m = Mandelbrot::new();
        let params = mandelbrot_params(3.0);
        assert!(!is_supported_fractal(&m, &params));
    }

    // ── ReferenceOrbit::compute_mandelbrot ───────────────────────────────

    #[test]
    fn orbit_at_origin_does_not_escape() {
        // c = 0+0i is in the Mandelbrot set (stays at 0 forever).
        let orbit = ReferenceOrbit::compute_mandelbrot(0.0, 0.0, 1.0, 256, 64);
        assert_eq!(orbit.orbit.len(), 257, "non-escaping reference stores z_0 through z_max_iter");
        // All entries should be (0, 0)
        for &(r, i) in &orbit.orbit {
            assert!(r.abs() < 1e-10 && i.abs() < 1e-10,
                "orbit at origin should stay at zero");
        }
    }

    #[test]
    fn orbit_at_exterior_escapes() {
        // c = 2+0i escapes immediately.
        let orbit = ReferenceOrbit::compute_mandelbrot(2.0, 0.0, 1.0, 1000, 64);
        assert!(orbit.orbit.len() < 1000, "orbit at c=2 should escape before max_iter");
    }

    #[test]
    fn orbit_is_valid_for_matching_params() {
        let orbit = ReferenceOrbit::compute_mandelbrot(0.5, 0.1, 1e12, 256, 64);
        assert!(orbit.is_valid_for(0.5, 0.1, 1e12, 256, 64));
    }

    #[test]
    fn orbit_invalid_for_different_zoom() {
        let orbit = ReferenceOrbit::compute_mandelbrot(0.5, 0.1, 1e12, 256, 64);
        assert!(!orbit.is_valid_for(0.5, 0.1, 2e12, 256, 64));
    }

    #[test]
    fn repeated_orbit_computation_is_deterministic() {
        let first = ReferenceOrbit::compute_mandelbrot(-0.743643887037151, 0.131825904205330, 1.55e13, 512, 128);
        let second = ReferenceOrbit::compute_mandelbrot(-0.743643887037151, 0.131825904205330, 1.55e13, 512, 128);

        assert_eq!(first.escaped, second.escaped);
        assert_eq!(first.orbit, second.orbit);
        assert_eq!(first.series_linear, second.series_linear);
        assert_eq!(first.series_quadratic, second.series_quadratic);
        assert_eq!(first.series_cubic, second.series_cubic);
    }

    #[test]
    fn orbit_at_period2_point_does_not_escape() {
        let orbit = ReferenceOrbit::compute_mandelbrot(-1.0, 0.0, 1.0e6, 256, 64);

        assert!(!orbit.escaped, "period-2 anchor should remain interior at this depth");
        assert_eq!(orbit.orbit.len(), 257, "non-escaping orbit should store z_0 through z_max_iter");
    }

    // ── pt_iterate ────────────────────────────────────────────────────────

    #[test]
    fn pt_iterate_pixel_at_center_agrees_with_reference_escape() {
        // dc = 0 means the pixel IS the reference. pt_iterate should return the
        // same escape iteration as the reference orbit length.
        let center_x = -0.75;
        let center_y = 0.0;
        let orbit = ReferenceOrbit::compute_mandelbrot(center_x, center_y, 1.0, 1000, 128);

        // At dc=0, the pixel orbit matches the reference exactly.
        let result = pt_iterate_from(&orbit, 0, 0.0, 0.0, 0.0, 0.0, 4.0, 1.0);
        match result {
            PtIterOutcome::Finished(n) => {
                // For a non-escaping reference, PT returns max_iter even though the
                // stored orbit includes the extra z_max_iter entry.
                let expected = if orbit.escaped {
                    orbit.orbit.len() as u32
                } else {
                    orbit.max_iter
                };
                assert_eq!(n, expected,
                    "center pixel should escape at same iteration as reference");
            }
            PtIterOutcome::NeedRebase(_) => panic!("center pixel should not require rebasing"),
        }
    }

    #[test]
    fn series_warm_start_matches_baseline_on_small_offset() {
        let orbit = ReferenceOrbit::compute_mandelbrot(0.0, 0.0, 1.0e16, 256, 128);
        let dc_re = 1.0e-16;
        let dc_im = -5.0e-17;

        let baseline = pt_iterate_from(&orbit, 0, 0.0, 0.0, dc_re, dc_im, 4.0, 1.0);
        let (skip, stats) = try_series_skip(&orbit, dc_re, dc_im, 4.0);
        let (skip_iteration, skip_dz_re, skip_dz_im) =
            skip.expect("expected conservative series skip");
        assert!(stats.accepted, "expected series stats to record an accepted warm start");
        assert!(skip_iteration > 0, "series skip should advance beyond the first iteration");

        let warmed = pt_iterate_from(
            &orbit,
            skip_iteration,
            skip_dz_re,
            skip_dz_im,
            dc_re,
            dc_im,
            4.0,
            1.0,
        );

        match (baseline, warmed) {
            (PtIterOutcome::Finished(a), PtIterOutcome::Finished(b)) => {
                assert_eq!(a, b, "series warm start changed the final iteration count");
            }
            _ => panic!("expected both paths to finish without rebasing"),
        }
    }

    #[test]
    fn series_warm_start_covers_max_skip_window() {
        let orbit = ReferenceOrbit::compute_mandelbrot(0.0, 0.0, 1.0e30, 2048, 128);
        let dc_re = 1.0e-30;
        let dc_im = -5.0e-31;

        let baseline = pt_iterate_from(&orbit, 0, 0.0, 0.0, dc_re, dc_im, 4.0, 1.0);
        let (skip, stats) = try_series_skip(&orbit, dc_re, dc_im, 4.0);
        let (skip_iteration, skip_dz_re, skip_dz_im) =
            skip.expect("expected deep SA window to accept a tiny offset around the origin");

        assert!(stats.accepted);
        assert_eq!(skip_iteration, PT_SERIES_MAX_SKIP.min(orbit.max_iter as usize));

        let warmed = pt_iterate_from(
            &orbit,
            skip_iteration,
            skip_dz_re,
            skip_dz_im,
            dc_re,
            dc_im,
            4.0,
            1.0,
        );

        match (baseline, warmed) {
            (PtIterOutcome::Finished(a), PtIterOutcome::Finished(b)) => {
                assert_eq!(a, b, "deep SA warm start changed the final iteration count");
            }
            _ => panic!("expected both deep SA paths to finish without rebasing"),
        }
    }

    #[test]
    fn series_skip_stays_disabled_near_escape() {
        let orbit = ReferenceOrbit::compute_mandelbrot(0.0, 0.0, 1.0, 256, 128);
        let (skip, stats) = try_series_skip(&orbit, 1.5, 0.0, 4.0);
        assert!(skip.is_none());
        assert!(stats.rejected_escape_margin);
    }

    #[test]
    fn render_surfaces_series_telemetry() {
        use crate::fractals::FractalView;

        let mut view = FractalView::new(16, 16);
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 1.0e16;

        let orbit = ReferenceOrbit::compute_mandelbrot(
            view.center_x,
            view.center_y,
            view.zoom,
            256,
            128,
        );
        let m = Mandelbrot::new();
        let params = mandelbrot_params(2.0);
        let result = render_perturbation(&view, &orbit, &m, &params, 256, 128, 0, 1.0);

        assert!(result.sa_accepted_pixel_count > 0, "expected SA to be accepted for at least one pixel");
        assert!(result.sa_total_skipped_iterations >= result.sa_accepted_pixel_count);
        assert!(result.sa_max_skipped_iterations > 0);
    }

    // ── render_perturbation ───────────────────────────────────────────────

    #[test]
    fn render_produces_correct_buffer_size() {
        use crate::fractals::FractalView;
        let view = FractalView::new(64, 64);
        let orbit = ReferenceOrbit::compute_mandelbrot(
            view.center_x, view.center_y, view.zoom, 256, 128
        );
        let m = Mandelbrot::new();
        let params = mandelbrot_params(2.0);
        let _result = render_perturbation(&view, &orbit, &m, &params, 256, 128, 0, 1.0);
    }

    #[test]
    fn render_at_low_zoom_sets_warning() {
        use crate::fractals::FractalView;
        let mut view = FractalView::new(32, 32);
        view.zoom = 1.0; // well below PT_LOW_ZOOM_THRESHOLD
        let orbit = ReferenceOrbit::compute_mandelbrot(
            view.center_x, view.center_y, view.zoom, 64, 64
        );
        let m = Mandelbrot::new();
        let params = mandelbrot_params(2.0);
        let result = render_perturbation(&view, &orbit, &m, &params, 64, 64, 0, 1.0);
        assert!(result.low_zoom_warning,
            "PT should surface a low-zoom warning instead of silently looking like the preferred path");
    }

    #[test]
    fn render_agrees_with_direct_f64_at_low_zoom() {
        // In a tame low-zoom interior window, PT and direct f64 should agree
        // exactly with no fallback.
        use crate::fractals::FractalView;
        use crate::rendering::compute_iterations;

        let mut view = FractalView::new(64, 64);
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 100.0;
        let orbit = ReferenceOrbit::compute_mandelbrot(
            view.center_x, view.center_y, view.zoom, 256, 128
        );
        let m = Mandelbrot::new();
        let params = mandelbrot_params(2.0);

        let pt_result = render_perturbation(&view, &orbit, &m, &params, 256, 128, 0, 1.0);
        let direct = compute_iterations(&view, 256, &m, &params);

        let mismatches: usize = pt_result.iterations.iter()
            .zip(direct.iter())
            .filter(|(a, b)| a != b)
            .count();

        assert_eq!(pt_result.glitch_count, 0,
            "PT should not need fallback in the tame low-zoom interior window");
        assert_eq!(mismatches, 0,
            "PT and direct f64 should agree on all pixels in the tame low-zoom interior window (mismatches: {})", mismatches);
    }

    #[test]
    fn render_agrees_with_direct_f64_at_mid_zoom() {
        use crate::rendering::compute_iterations;

        let view = scene_view(-1.0, 0.0, 1.0e6, 32, 32);
        let orbit = ReferenceOrbit::compute_mandelbrot(
            view.center_x,
            view.center_y,
            view.zoom,
            512,
            128,
        );
        let m = Mandelbrot::new();
        let params = mandelbrot_params(2.0);

        let pt_result = render_perturbation(&view, &orbit, &m, &params, 512, 128, 0, 1.0);
        let direct = compute_iterations(&view, 512, &m, &params);

        assert_eq!(pt_result.glitch_count, 0,
            "period-2 mid-zoom scene should remain on the PT delta path");
        assert_eq!(pt_result.iterations, direct,
            "PT and direct f64 diverged on the representative mid-zoom period-2 scene");
    }

    #[test]
    fn known_point_regression_matches_recorded_iteration() {
        let orbit = ReferenceOrbit::compute_mandelbrot(0.5, 0.5, 1.0, 64, 128);

        match pt_iterate_from(&orbit, 0, 0.0, 0.0, 0.0, 0.0, 4.0, 1.0) {
            PtIterOutcome::Finished(iterations) => {
                assert_eq!(iterations, 4,
                    "the recorded Mandelbrot regression point c=0.5+0.5i should escape after 4 iterations");
            }
            PtIterOutcome::NeedRebase(_) => {
                panic!("known regression point at the reference center should not require rebasing");
            }
        }
    }

    #[test]
    fn render_wrapper_matches_single_tile_tiled_render() {
        let view = scene_view(-0.743643887037151, 0.131825904205330, 1.55e13, 32, 18);
        let orbit = ReferenceOrbit::compute_mandelbrot(
            view.center_x,
            view.center_y,
            view.zoom,
            4096,
            128,
        );
        let m = Mandelbrot::new();
        let params = mandelbrot_params(2.0);

        let wrapper = render_perturbation(&view, &orbit, &m, &params, 4096, 128, 0, 1.0);
        let tiled = render_perturbation_tiled(
            &view,
            std::slice::from_ref(&orbit),
            1,
            1,
            &m,
            &params,
            4096,
            128,
            0,
            1.0,
        );

        assert_eq!(wrapper.iterations, tiled.iterations);
        assert_eq!(wrapper.glitch_count, tiled.glitch_count);
        assert_eq!(wrapper.glitch_mask, tiled.glitch_mask);
        assert_eq!(wrapper.rebase_count, tiled.rebase_count);
        assert_eq!(wrapper.rebased_pixel_count, tiled.rebased_pixel_count);
        assert_eq!(wrapper.rebase_exhausted_count, tiled.rebase_exhausted_count);
        assert_eq!(wrapper.sa_accepted_pixel_count, tiled.sa_accepted_pixel_count);
        assert_eq!(wrapper.sa_total_skipped_iterations, tiled.sa_total_skipped_iterations);
        assert_eq!(wrapper.sa_max_skipped_iterations, tiled.sa_max_skipped_iterations);
        assert_eq!(wrapper.sa_rejected_escape_margin_count, tiled.sa_rejected_escape_margin_count);
        assert_eq!(wrapper.sa_rejected_correction_count, tiled.sa_rejected_correction_count);
        assert_eq!(wrapper.low_zoom_warning, tiled.low_zoom_warning);
    }

    #[test]
    fn glitch_count_is_surfaced_not_hidden() {
        // A trivially constructed orbit with a single entry at (0,0) will
        // cause every pixel (except dc=0) to be flagged as glitch because
        // the orbit is exhausted before pixels escape. This verifies the
        // glitch counter works.
        use crate::fractals::FractalView;
        let view = FractalView::new(16, 16);

        // Manually build an orbit that terminates after 1 step at a point
        // that will cause most pixels to glitch via the "reference escaped
        // but pixel did not" path.
        let orbit = ReferenceOrbit {
            orbit: vec![(0.0, 0.0)],  // length 1, max_iter=1000, escaped=true → glitch path
            escaped: true,
            center_x: view.center_x,
            center_y: view.center_y,
            zoom: view.zoom,
            max_iter: 1000,
            bits: 128,
            tile_frac_x: 0.5,
            tile_frac_y: 0.5,
            series_linear: vec![(0.0, 0.0)],
            series_quadratic: vec![(0.0, 0.0)],
            series_cubic: vec![(0.0, 0.0)],
        };

        let m = Mandelbrot::new();
        let params = mandelbrot_params(2.0);
        let result = render_perturbation(&view, &orbit, &m, &params, 1000, 128, 0, 1.0);

        // ALL pixels except dc=0 should be glitches (fallback to direct f64).
        // dc=0 (center pixel) hits escape check at iteration 0: |0+0|^2 < 4,
        // no glitch, orbit exhausted → glitch via "ref escaped early" path.
        assert!(result.glitch_count > 0,
            "truncated orbit should produce glitches; got 0");
    }

    #[test]
    fn mismatched_reference_triggers_glitches() {
        let orbit = ReferenceOrbit::compute_mandelbrot(1.0, 0.0, 1.0, 64, 64);

        match pt_iterate_from(&orbit, 0, 0.0, 0.0, -1.0, 0.0, 4.0, 1.0) {
            PtIterOutcome::NeedRebase(point) => {
                assert_eq!(point.iteration, 1,
                    "synthetic mismatched reference should trigger the glitch detector on the first meaningful step");
            }
            PtIterOutcome::Finished(iterations) => panic!(
                "expected a synthetic mismatched reference to trigger rebasing, got Finished({iterations})"
            ),
        }
    }

    #[test]
    fn fallback_path_matches_direct_f64_when_forced() {
        use crate::rendering::compute_iterations;

        let view = crate::fractals::FractalView::new(16, 16);
        let forced_fallback_orbit = ReferenceOrbit {
            orbit: vec![(0.0, 0.0)],
            escaped: true,
            center_x: view.center_x,
            center_y: view.center_y,
            zoom: view.zoom,
            max_iter: 256,
            bits: 128,
            tile_frac_x: 0.5,
            tile_frac_y: 0.5,
            series_linear: vec![(0.0, 0.0)],
            series_quadratic: vec![(0.0, 0.0)],
            series_cubic: vec![(0.0, 0.0)],
        };

        let m = Mandelbrot::new();
        let params = mandelbrot_params(2.0);
        let result = render_perturbation(&view, &forced_fallback_orbit, &m, &params, 256, 64, 0, 1.0);
        let direct = compute_iterations(&view, 256, &m, &params);

        assert!(result.glitch_count > 0,
            "forced truncated orbit should route at least some pixels through the fallback path");
        for ((actual, expected), glitch) in result.iterations.iter().zip(direct.iter()).zip(result.glitch_mask.iter()) {
            if *glitch {
                assert_eq!(actual, expected,
                    "fallback pixels should match direct f64 iteration counts exactly");
            }
        }
    }

    #[test]
    fn scratch_scene_regression_keeps_fallback_visible() {
        let view = scene_view(-1.2541275710056559, 0.38365671509396854, 1.55e13, 64, 36);
        let orbit = ReferenceOrbit::compute_mandelbrot(
            view.center_x,
            view.center_y,
            view.zoom,
            4096,
            128,
        );
        let scale = 3.5 / view.zoom;
        let aspect = view.width as f64 / view.height as f64;
        let range_x = scale * aspect;
        let range_y = scale;
        let mut saw_primary_fallback = false;

        'pixels: for py in 0..view.height as usize {
            for px in 0..view.width as usize {
                let outcome = pt_outcome_for_pixel(
                    px,
                    py,
                    view.width as usize,
                    view.height as usize,
                    range_x,
                    range_y,
                    std::slice::from_ref(&orbit),
                    1,
                    1,
                    4.0,
                    1.0,
                );

                if matches!(outcome, PtPixelOutcome::NeedsFallback { .. }) {
                    saw_primary_fallback = true;
                    break 'pixels;
                }
            }
        }

        assert!(saw_primary_fallback,
            "the scratch regression scene should still produce primary-path PT fallback pressure instead of silently looking easy");
    }

    #[test]
    fn denser_retry_grid_recovers_at_least_one_primary_fallback_pixel() {
        let view = scene_view(-0.743643887037151, 0.131825904205330, 1.55e13, 64, 36);
        let max_iter = 4096;
        let bits = 128;
        let primary_tiles = 4usize;
        let retry_tiles = 8usize;
        let primary_orbits = ReferenceOrbit::compute_tile_orbits(&view, primary_tiles as u32, max_iter, bits);
        let retry_orbits = ReferenceOrbit::compute_tile_orbits(&view, retry_tiles as u32, max_iter, bits);
        let scale = 3.5 / view.zoom;
        let aspect = view.width as f64 / view.height as f64;
        let range_x = scale * aspect;
        let range_y = scale;
        let mut recovered = false;

        'pixels: for py in 0..view.height as usize {
            for px in 0..view.width as usize {
                let primary = pt_outcome_for_pixel(
                    px,
                    py,
                    view.width as usize,
                    view.height as usize,
                    range_x,
                    range_y,
                    &primary_orbits,
                    primary_tiles,
                    primary_tiles,
                    4.0,
                    1.0,
                );

                if !matches!(primary, PtPixelOutcome::NeedsFallback { .. }) {
                    continue;
                }

                let retry = pt_outcome_for_pixel(
                    px,
                    py,
                    view.width as usize,
                    view.height as usize,
                    range_x,
                    range_y,
                    &retry_orbits,
                    retry_tiles,
                    retry_tiles,
                    4.0,
                    1.0,
                );

                if matches!(retry, PtPixelOutcome::Finished { .. }) {
                    recovered = true;
                    break 'pixels;
                }
            }
        }

        assert!(recovered,
            "expected at least one boundary-scene pixel where the denser retry grid succeeds after the primary tiled PT path falls back");
    }
}
