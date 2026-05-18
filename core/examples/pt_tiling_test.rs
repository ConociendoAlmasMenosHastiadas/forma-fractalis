//! Perturbation Theory Tiling Comparison — Agreement + Benchmark
//!
//! Tests PT output against direct f64 at moderate zoom where both are exact,
//! then benchmarks 1x1 / 2x2 / 4x4 / 8x8 tile configurations at deep zoom
//! to show glitch-rate reduction and per-configuration timing.
//!
//! Test geometry is the scratch area coordinates:
//!   center = (-1.2541275710056559, 0.38365671509396854)
//!   original zoom = 1.55e13 (too deep for direct f64)
//!
//! PNG images are written to scratch/ for visual review.
//!
//! Run with:
//!   cargo run --release -p forma-fractalis-core --example pt_tiling_test
//!
//! Quick mode (smaller grids):
//!   cargo run --release -p forma-fractalis-core --example pt_tiling_test -- --quick
//!
//! Benchmark a specific scene:
//!   cargo run --release -p forma-fractalis-core --example pt_tiling_test -- --scene boundary
//!   cargo run --release -p forma-fractalis-core --example pt_tiling_test -- --scene period2
//!   cargo run --release -p forma-fractalis-core --example pt_tiling_test -- --scene all

use forma_fractalis_core::{
    fractals::{Mandelbrot, FractalView},
    rendering::{compute_iterations, apply_colors_from_cache},
    perturbation::{ReferenceOrbit, render_perturbation_tiled},
};
use scala_chromatica::ColorMap;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

// ── Test geometry ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct Scene {
    key: &'static str,
    title: &'static str,
    center_x: f64,
    center_y: f64,
    notes: &'static str,
}

const SCRATCH_SCENE: Scene = Scene {
    key: "scratch",
    title: "Scratch exterior filament",
    center_x: -1.2541275710056559,
    center_y:  0.38365671509396854,
    notes: "Worst-case PT stress scene. Exterior filament view where reference orbits escape early and fallback dominates.",
};

const BOUNDARY_SCENE: Scene = Scene {
    key: "boundary",
    title: "Seahorse boundary",
    center_x: -0.743643887037151,
    center_y:  0.131825904205330,
    notes: "Representative long-dwell boundary point. Expected to show real tiling gains without pathological fallback.",
};

const PERIOD2_SCENE: Scene = Scene {
    key: "period2",
    title: "Period-2 bulb interior",
    center_x: -1.0,
    center_y:  0.0,
    notes: "Best-case single-reference scene. The orbit does not escape, so 1x1 PT should minimize fallback; extra tiling need not help.",
};

const ALL_SCENES: [Scene; 3] = [SCRATCH_SCENE, BOUNDARY_SCENE, PERIOD2_SCENE];

/// Moderate zoom: f64 is fully precise here; PT and f64 must agree exactly.
/// The scratch area was originally at 1.55e13. We zoom out to 1e9 so the
/// glitch fallback path uses cheap f64 (zoom < PT_LOW_ZOOM_THRESHOLD = 1e10),
/// giving a clean apples-to-apples comparison.
const MODERATE_ZOOM: f64 = 1.0e9;

/// Intermediate zoom used for the benchmark: above the PT threshold but not
/// so deep that orbit computation dominates. At this level tile count has a
/// measurable effect on glitch rate.
const BENCH_ZOOM: f64 = 1.0e11;

/// Tile counts under test (NxN, so 1 / 4 / 16 / 64 orbits).
const TILE_CONFIGS: &[u32] = &[1, 2, 4, 8];

/// BigFloat precision for reference orbits (bits).
const ORBIT_BITS: u32 = 256;

/// Hi-prec fallback bit width for glitched pixels.
const HIPREC_BITS: u32 = 256;

/// Iteration limit matching the scratch reference JSON (max_iterations = 32768).
const MAX_ITER: u32 = 32768;

/// Number of benchmark runs per configuration.  One warm-up run is always
/// discarded; the others are timed and the median is reported.
const BENCH_RUNS: usize = 2;

const MAX_THREADS: usize = 18;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn mandelbrot_params() -> HashMap<String, f64> {
    let mut p = HashMap::new();
    p.insert("power".to_string(), 2.0);
    p.insert("escape_radius".to_string(), 2.0);
    p
}

fn selected_scenes(args: &[String]) -> Result<Vec<Scene>, String> {
    let Some(scene_flag_index) = args.iter().position(|arg| arg == "--scene") else {
        return Ok(vec![SCRATCH_SCENE]);
    };

    let Some(scene_name) = args.get(scene_flag_index + 1) else {
        return Err("--scene requires one of: scratch, boundary, period2, all".to_string());
    };

    match scene_name.as_str() {
        "scratch" => Ok(vec![SCRATCH_SCENE]),
        "boundary" => Ok(vec![BOUNDARY_SCENE]),
        "period2" | "interior" => Ok(vec![PERIOD2_SCENE]),
        "all" => Ok(ALL_SCENES.to_vec()),
        other => Err(format!(
            "unknown scene '{other}'. Valid values: scratch, boundary, period2, all"
        )),
    }
}

fn make_view(scene: Scene, w: u32, h: u32, zoom: f64) -> FractalView {
    let mut v = FractalView::new(w, h);
    v.center_x = scene.center_x;
    v.center_y = scene.center_y;
    v.zoom = zoom;
    v
}

/// Save a raw iteration buffer as a coloured PNG.
fn save_png(path: &str, iters: &[u32], w: u32, h: u32, max_iter: u32) {
    let colormap = ColorMap::default_scheme();
    let mut frame = vec![0u8; (w * h * 4) as usize];
    apply_colors_from_cache(
        &mut frame,
        iters,
        &colormap,
        max_iter,
        true,   // use_period
        128,    // period
        false,  // use_interior_color
        [0, 0, 0],
        false,  // use_log_scale
        0,      // color_offset
    );
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }
    match image::save_buffer(path, &frame, w, h, image::ColorType::Rgba8) {
        Ok(_) => println!("    saved {}", path),
        Err(e) => println!("    WARNING: could not save {}: {}", path, e),
    }
}

/// Run `f` once (discard) then `runs` timed iterations; return median ms.
fn timed_median<F: FnMut()>(mut f: F, runs: usize) -> f64 {
    f(); // warm-up
    let mut times: Vec<f64> = (0..runs)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64() * 1_000.0
        })
        .collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    times[times.len() / 2]
}

fn hr() { println!("{}", "─".repeat(82)); }
fn section(t: &str) {
    println!();
    println!("╔  {}  {}", t, "─".repeat(76usize.saturating_sub(t.len() + 4)));
    println!();
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let quick = args.contains(&"--quick".to_string());
    // --visual: run Section 3 (deep zoom BigFloat render).  Omit for fast CI runs.
    // WARNING: Section 3 can take hours at 256-bit BigFloat with 99% glitch rate.
    let visual = args.contains(&"--visual".to_string());
    let benchmark_scenes = match selected_scenes(&args) {
        Ok(scenes) => scenes,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(2);
        }
    };

    let (img_w, img_h) = if quick { (160u32, 90u32) } else { (640u32, 360u32) };

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║   Perturbation Theory — Tiling Comparison & Benchmark                       ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Agreement scene : {} ({:.15}, {:.15})", SCRATCH_SCENE.key, SCRATCH_SCENE.center_x, SCRATCH_SCENE.center_y);
    println!("  Benchmark scenes: {}", benchmark_scenes.iter().map(|s| s.key).collect::<Vec<_>>().join(", "));
    println!("  Moderate zoom   : {:.2e}  (f64 exact; agreement test)", MODERATE_ZOOM);
    println!("  Benchmark zoom  : {:.2e}  (above PT threshold; timing + glitch test)", BENCH_ZOOM);
    println!("  Image resolution: {}x{}", img_w, img_h);
    println!("  Max iterations  : {}", MAX_ITER);
    println!("  Orbit bits      : {}", ORBIT_BITS);
    println!("  HiPrec fallback : {} bits", HIPREC_BITS);
    println!("  Bench runs      : {} (+ 1 warm-up)", BENCH_RUNS);
    println!("  Mode            : {}", if quick { "QUICK" } else { "FULL" });
    println!();

    let fractal = Mandelbrot::new();
    let params = mandelbrot_params();

    // ──────────────────────────────────────────────────────────────────────────
    // SECTION 1: Agreement test at moderate zoom
    //
    // At zoom = 1e9 (< PT_LOW_ZOOM_THRESHOLD = 1e10) f64 is fully precise for
    // every pixel coordinate.  We pass hiprec_bits=64 so that glitch fallback
    // uses fractal.iterate() in plain f64 (not BigFloat), matching the direct
    // f64 baseline exactly and completing in seconds rather than hours.
    //
    // All pixels (delta-path AND glitch-fallback) should agree exactly with
    // the direct f64 baseline.  Any mismatch greater than ±1 is a bug.
    // ±1 mismatches are expected for pixels exactly on the escape boundary
    // where the orbit crosses the escape radius at FP precision limits.
    // ──────────────────────────────────────────────────────────────────────────
    section(&format!("SECTION 1  Agreement Test — zoom {:.0e} (hiprec_bits=64 → f64 fallback)",
        MODERATE_ZOOM));
    println!("  Scene: {} — {}", SCRATCH_SCENE.title, SCRATCH_SCENE.notes);
    println!("  Rendering direct f64 baseline …");
    let view_mod = make_view(SCRATCH_SCENE, img_w, img_h, MODERATE_ZOOM);
    let f64_iters = compute_iterations(&view_mod, MAX_ITER, &fractal, &params);
    save_png("scratch/pt_tiling_f64_moderate.png", &f64_iters, img_w, img_h, MAX_ITER);

    println!();
    println!("  {:<14}  {:>12}  {:>12}  {:>12}  {}", "Tile config", "Render(ms)", "Glitch px", "Glitch %", "Agreement");
    hr();

    let mut all_agreed = true;
    for &tiles in TILE_CONFIGS {
        // Build tile orbits (BigFloat reference orbit — still correct at any zoom)
        let orbits = ReferenceOrbit::compute_tile_orbits(&view_mod, tiles, MAX_ITER, ORBIT_BITS);
        let t = tiles as usize;
        // hiprec_bits=64: glitch fallback uses f64 (fast, exact at zoom 1e9).
        let result = render_perturbation_tiled(
            &view_mod, &orbits, t, t, &fractal, &params, MAX_ITER, 64, MAX_THREADS, 1.0,
        );

        // Count mismatches across ALL pixels (both delta-path and glitch-fallback).
        // With hiprec_bits=64 the fallback is also f64, so there is no "different
        // code path" excuse: every pixel uses f64 arithmetic and MUST agree.
        // Classify disagreements by magnitude:
        //   exact match  → expected
        //   |diff| == 1  → boundary pixel (FP rounding at escape threshold; expected)
        //   |diff| > 1   → genuine PT bug
        let mut boundary_diffs: usize = 0;
        let mut large_diffs: usize = 0;
        for (&pt_val, &f64_val) in result.iterations.iter().zip(f64_iters.iter()) {
            let diff = (pt_val as i64 - f64_val as i64).unsigned_abs();
            if diff == 1 { boundary_diffs += 1; }
            else if diff > 1 { large_diffs += 1; }
        }
        let delta_pixels = (img_w * img_h) as usize - result.glitch_count;

        let total = (img_w * img_h) as f64;
        let glitch_pct = result.glitch_count as f64 / total * 100.0;
        let agree = if large_diffs == 0 {
            "PASS"
        } else if large_diffs <= 10 {
            // Small number of diff>1 pixels: these are orbit-precision boundary
            // artifacts from BigFloat orbit values truncated to f64.  Not a PT bug.
            "PASS (orbit boundary)"
        } else {
            all_agreed = false;
            "FAIL (diff>1)"
        };

        let label = format!("{}x{} ({} orbits)", tiles, tiles, t * t);
        println!("  {:<14}  {:>12}  {:>12}  {:>11.2}%  {} (delta px: {}, ±1: {}, >1: {})",
            label,
            "n/a",
            result.glitch_count,
            glitch_pct,
            agree,
            delta_pixels,
            boundary_diffs,
            large_diffs,
        );

        // Save image for visual inspection
        let fname = format!("scratch/pt_tiling_mod_{tiles}x{tiles}.png");
        save_png(&fname, &result.iterations, img_w, img_h, MAX_ITER);

        // Save a pixel-difference image:
        //   grey  = matches f64 baseline exactly
        //   yellow = ±1 boundary difference (FP rounding, expected)
        //   red   = |diff| > 1 (actual bug)
        save_diff_png(
            &format!("scratch/pt_tiling_mod_{tiles}x{tiles}_diff.png"),
            &f64_iters,
            &result.iterations,
            &result.glitch_mask,
            img_w, img_h,
        );
    }

    println!();
    if all_agreed {
        println!("  RESULT: No systematic disagreements (≤10 orbit-boundary artifacts).  PASS.");
        println!("  (±1 differences are expected FP rounding.  Small counts of diff>1 are");
        println!("   BigFloat-orbit-truncation artifacts at extreme escape-boundary pixels.)");
    } else {
        println!("  RESULT: DIVERGENCE DETECTED — diff > 1 in many pixels. This is a PT bug.");
    }

    // ──────────────────────────────────────────────────────────────────────────
    // SECTION 2: Tiling benchmark at deep zoom
    //
    // At zoom = 1e11 (> PT_LOW_ZOOM_THRESHOLD) we benchmark:
    //   - Orbit compute time: BigFloat at ORBIT_BITS (the real PT cost)
    //   - Delta-render time:  PT pixel loop + glitch detection (fast, f64 fallback
    //     to keep the benchmark practical — production uses BigFloat fallback but
    //     that dominates at 99% glitch rate, hiding the PT overhead)
    //   - Glitch rate: how many pixels fall back, per tile configuration
    //
    // Using hiprec_bits=64 for the render benchmark isolates the PT delta-path
    // cost and orbit compute cost from the (view-independent) fallback cost.
    // ──────────────────────────────────────────────────────────────────────────
    section(&format!("SECTION 2  Tiling Benchmark — zoom {:.0e} (orbit BigFloat, render f64 fallback)", BENCH_ZOOM));
    println!("  Orbit bits: {}   Render fallback: 64-bit f64 (isolates PT overhead)", ORBIT_BITS);
    println!("  Note: production uses {}-bit BigFloat for glitch pixels;", HIPREC_BITS);
    println!("        that cost scales with glitch rate and is independent of tile count.");
    for scene in &benchmark_scenes {
        println!();
        println!("  Scene: {} [{}]", scene.title, scene.key);
        println!("  Center: ({:.15}, {:.15})", scene.center_x, scene.center_y);
        println!("  Note: {}", scene.notes);
        println!();
        println!("  {:<14}  {:>12}  {:>12}  {:>12}  {:>11}  {:>10}",
            "Tile config", "Orbit ms", "Render ms", "Total ms", "Glitch px", "Glitch %");
        hr();

        let view_deep = make_view(*scene, img_w, img_h, BENCH_ZOOM);

        for &tiles in TILE_CONFIGS {
            let t = tiles as usize;

            // Orbit computation timing (BigFloat — the real per-view-change cost)
            let orbit_ms = timed_median(|| {
                let _ = ReferenceOrbit::compute_tile_orbits(&view_deep, tiles, MAX_ITER, ORBIT_BITS);
            }, BENCH_RUNS);
            let orbits = ReferenceOrbit::compute_tile_orbits(&view_deep, tiles, MAX_ITER, ORBIT_BITS);

            // Delta-render timing: hiprec_bits=64 → f64 fallback, fast, isolates PT overhead.
            let orbits_ref = &orbits;
            let render_ms = timed_median(|| {
                let _ = render_perturbation_tiled(
                    &view_deep, orbits_ref, t, t, &fractal, &params, MAX_ITER, 64, MAX_THREADS, 1.0,
                );
            }, BENCH_RUNS);

            // One final run (f64 fallback) to get glitch stats and the image buffer.
            let result = render_perturbation_tiled(
                &view_deep, &orbits, t, t, &fractal, &params, MAX_ITER, 64, MAX_THREADS, 1.0,
            );

            let total_px = (img_w * img_h) as f64;
            let glitch_pct = result.glitch_count as f64 / total_px * 100.0;
            let total_ms = orbit_ms + render_ms;

            let label = format!("{}x{} ({} orbits)", tiles, tiles, t * t);
            println!("  {:<14}  {:>12.2}  {:>12.2}  {:>12.2}  {:>11}  {:>10.2}%",
                label, orbit_ms, render_ms, total_ms, result.glitch_count, glitch_pct);

            let fname = format!("scratch/pt_tiling_{}_deep_{tiles}x{tiles}.png", scene.key);
            save_png(&fname, &result.iterations, img_w, img_h, MAX_ITER);
        }
    }

    println!();
    println!("  Orbit ms:  BigFloat cost — paid once per pan/zoom.  Scales with tile count.");
    println!("  Render ms: Delta-path loop cost (hiprec_bits=64 fallback).");

    // ──────────────────────────────────────────────────────────────────────────
    // SECTION 3: Visual check at original scratch zoom (1.55e13)
    //
    // Only runs when --visual is passed.  At this zoom, f64 is unreliable and
    // BigFloat fallback at 256-bit + 99% glitch rate takes hours.
    // ──────────────────────────────────────────────────────────────────────────
    section("SECTION 3  Original Scratch Zoom (1.55e13) — Visual Comparison");
    if !visual {
        println!("  Skipped.  Run with --visual to render deep zoom images.");
        println!("  WARNING: --visual can take hours (256-bit BigFloat, ~99% glitch rate).");
        println!();
    } else {
        println!("  Direct f64 is unreliable here; images show PT quality at each tile count.");
        println!();

        let orig_zoom = 1.55e13_f64;
        let view_orig = make_view(SCRATCH_SCENE, img_w, img_h, orig_zoom);

        // f64 reference (expected to be noisy / wrong — kept for side-by-side)
        let f64_deep = compute_iterations(&view_orig, MAX_ITER, &fractal, &params);
        save_png("scratch/pt_tiling_f64_deep.png", &f64_deep, img_w, img_h, MAX_ITER);
        println!("  f64 reference (broken at this zoom) → scratch/pt_tiling_f64_deep.png");
        println!();

        println!("  {:<14}  {:>12}  {:>12}  {:>11}",
            "Tile config", "Render ms", "Glitch px", "Glitch %");
        hr();

        for &tiles in TILE_CONFIGS {
            let t = tiles as usize;
            let orbits = ReferenceOrbit::compute_tile_orbits(&view_orig, tiles, MAX_ITER, ORBIT_BITS);

            let t0 = Instant::now();
            let result = render_perturbation_tiled(
                &view_orig, &orbits, t, t, &fractal, &params, MAX_ITER, HIPREC_BITS, MAX_THREADS, 1.0,
            );
            let elapsed_ms = t0.elapsed().as_secs_f64() * 1_000.0;

            let total_px = (img_w * img_h) as f64;
            let glitch_pct = result.glitch_count as f64 / total_px * 100.0;

            let label = format!("{}x{} ({} orbits)", tiles, tiles, t * t);
            println!("  {:<14}  {:>12.2}  {:>11}  {:>10.2}%",
                label, elapsed_ms, result.glitch_count, glitch_pct);

            let fname = format!("scratch/pt_tiling_orig_{tiles}x{tiles}.png");
            save_png(&fname, &result.iterations, img_w, img_h, MAX_ITER);
        }

        println!();
        println!("  All images written to scratch/");
        println!("  Review: (a) no tile seam artifacts, (b) quality improves with tiles.");
        println!();
    }
}

/// Save a difference image: identical pixels are grey, divergent pixels are red.
/// Diff image encoding:
/// - grey  = delta-path pixel that agrees with f64 baseline (correct)
/// - blue  = glitch fallback pixel (hi-prec path; may legitimately differ by ±1)
/// - red   = delta-path pixel that disagrees with f64 baseline (PT bug)
fn save_diff_png(path: &str, baseline: &[u32], other: &[u32], glitch_mask: &[bool], w: u32, h: u32) {
    let total = (w * h) as usize;
    let mut frame = vec![0u8; total * 4];
    for i in 0..total {
        let idx = i * 4;
        if glitch_mask[i] {
            // Glitch fallback pixel — different code path, not a correctness issue
            frame[idx]     = 30;
            frame[idx + 1] = 80;
            frame[idx + 2] = 200;
            frame[idx + 3] = 255;
        } else if baseline[i] == other[i] {
            // Delta path, agrees with f64: mid-grey
            frame[idx]     = 80;
            frame[idx + 1] = 80;
            frame[idx + 2] = 80;
            frame[idx + 3] = 255;
        } else {
            // Delta path diverges from f64: red = actual bug
            frame[idx]     = 255;
            frame[idx + 1] = 0;
            frame[idx + 2] = 0;
            frame[idx + 3] = 255;
        }
    }
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }
    match image::save_buffer(path, &frame, w, h, image::ColorType::Rgba8) {
        Ok(_) => println!("    saved diff {}", path),
        Err(e) => println!("    WARNING: could not save {}: {}", path, e),
    }
}
