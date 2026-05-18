//! Perturbation Theory vs Hi-Precision CPU — Goldilocks Benchmark
//!
//! Finds the optimal rendering mode at each zoom level by measuring:
//!   - Wall-clock render time (median of N runs)
//!   - Glitch rate for PT (fraction of pixels re-rendered via hi-prec fallback)
//!   - Pixel-level accuracy vs ground-truth HiPrec-1024 (small quality patch)
//!
//! Test geometry is the debug coordinates shipped in scratch/:
//!   center = (-1.2541275710056559, 0.38365671509396854)
//!   zoom = 1.55e13 (deep zoom, f64 is broken here)
//!
//! Run:
//!   cargo bench --bench perturbation_bench -p forma-fractalis-core
//!
//! Quick smoke run (reduces resolution, fewer runs):
//!   cargo bench --bench perturbation_bench -p forma-fractalis-core -- --quick
//!
//! Deep scene matrix only (CPU f64 vs Hi-Prec vs PT, across multiple scenes):
//!   cargo bench --bench perturbation_bench -p forma-fractalis-core -- --quick --matrix-only
//!
//! Run the matrix at the intended GUI preview resolution:
//!   cargo bench --bench perturbation_bench -p forma-fractalis-core -- --quick --matrix-only --preview-res 1280x720

use forma_fractalis_core::{
    fractals::{Mandelbrot, FractalView},
    rendering::{compute_iterations, compute_iterations_hiprec},
    perturbation::{ReferenceOrbit, render_perturbation, render_perturbation_tiled, PT_LOW_ZOOM_THRESHOLD, PT_REFERENCE_BITS},
};
use std::collections::HashMap;
use std::time::Instant;

// --- Test geometry ----------------------------------------------------------

#[derive(Clone, Copy)]
struct Scene {
    key: &'static str,
    center_x: f64,
    center_y: f64,
    notes: &'static str,
}

/// The deep-zoom test center (from scratch/fractal_settings_mandel_debug.json).
const CENTER_X: f64 = -1.2541275710056559;
const CENTER_Y: f64 = 0.38365671509396854;

const SCRATCH_SCENE: Scene = Scene {
    key: "scratch",
    center_x: CENTER_X,
    center_y: CENTER_Y,
    notes: "Deep exterior filament scene. Useful for comparing naive 1x1 PT against tiled PT; behavior depends strongly on tile count.",
};

const BOUNDARY_SCENE: Scene = Scene {
    key: "boundary",
    center_x: -0.743643887037151,
    center_y: 0.131825904205330,
    notes: "Representative long-dwell boundary point. PT may still spend most of its time in fallback here even with tiling.",
};

const PERIOD2_SCENE: Scene = Scene {
    key: "period2",
    center_x: -1.0,
    center_y: 0.0,
    notes: "Best-case single-reference scene. 1x1 PT should already be near ideal; extra tiling need not help.",
};

const SCENE_MATRIX_SCENES: &[Scene] = &[SCRATCH_SCENE, BOUNDARY_SCENE, PERIOD2_SCENE];

const SCENE_MATRIX_ZOOM: f64 = 1.55e13;
const SCENE_MATRIX_ITER: u32 = 4096;
const SCENE_MATRIX_PT_TILES: &[u32] = &[1, 4];

/// Available bit widths for BigFloat hi-prec paths.
const BIT_WIDTHS: &[u32] = &[64, 128, 256, 512, 1024];

/// PT reference-orbit bit widths to test.
const PT_ORBIT_BITS: &[u32] = &[64, 128, 256, 512];

/// Zoom levels for the coarse sweep — just PT vs f64, fast.
const COARSE_ZOOM_SWEEP: &[f64] = &[
    1.0e9,
    1.0e10,
    1.0e11,
    5.0e11,
    1.0e12,
    5.0e12,
    1.55e13,
    1.0e14,
    1.0e15,
];

/// Fine-grained zoom range around the PT crossover (1e12–1e14).
const FINE_ZOOM_CROSSOVER: &[f64] = &[
    1.0e12, 2.0e12, 5.0e12,
    1.0e13, 1.55e13, 3.0e13,
    1.0e14,
];

/// Iteration counts for the scaling section.
/// Note: 32768 excluded because at 99%+ glitch rates the PT run degenerates into a
/// near-full HiPrec render, which takes many minutes and provides no new information.
const ITER_SWEEP: &[u32] = &[512, 1024, 4096, 8192];

// --- Thread limit -----------------------------------------------------------

/// Keep the benchmark from eating all cores (matches user preference).
const MAX_THREADS: usize = 18;

// --- Helpers ----------------------------------------------------------------

fn mandelbrot_params() -> HashMap<String, f64> {
    let mut p = HashMap::new();
    p.insert("power".to_string(), 2.0);
    p.insert("escape_radius".to_string(), 2.0);
    p
}

/// Run `f` for `warmup` discarded runs then `runs` measured runs.
/// Returns the median elapsed time in milliseconds.
fn timed_median<F: FnMut()>(mut f: F, warmup: usize, runs: usize) -> f64 {
    for _ in 0..warmup {
        f();
    }
    let mut times: Vec<f64> = (0..runs)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    times[times.len() / 2]
}

/// Compute a ground-truth iteration buffer at HiPrec-1024.
fn ground_truth(view: &FractalView, max_iter: u32, fractal: &Mandelbrot) -> Vec<u32> {
    let params = mandelbrot_params();
    compute_iterations_hiprec(view, max_iter, fractal, &params, 1024, MAX_THREADS)
        .expect("HiPrec-1024 should always succeed for Mandelbrot")
}

/// Count pixels where `a` and `b` differ.  Returns (mismatch_count, total).
fn pixel_diff(a: &[u32], b: &[u32]) -> (usize, usize) {
    let mismatches = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
    (mismatches, a.len())
}

/// Format speedup relative to a baseline (higher is faster).
fn speedup(method_ms: f64, baseline_ms: f64) -> String {
    if baseline_ms <= 0.0 {
        return "    -   ".to_string();
    }
    let ratio = baseline_ms / method_ms;
    if ratio >= 1.0 {
        format!("{:5.2}x faster", ratio)
    } else {
        format!("{:5.2}x SLOWER", 1.0 / ratio)
    }
}

fn hr(width: usize) {
    println!("{}", "─".repeat(width));
}

fn section(title: &str) {
    println!();
    println!("┌─ {} {}", title, "─".repeat(75usize.saturating_sub(title.len() + 3)));
    println!();
}

fn parse_resolution(value: &str) -> Result<(u32, u32), String> {
    let Some((width, height)) = value.split_once(['x', 'X']) else {
        return Err(format!("resolution '{value}' must be WIDTHxHEIGHT"));
    };

    let width = width
        .parse::<u32>()
        .map_err(|_| format!("invalid width in resolution '{value}'"))?;
    let height = height
        .parse::<u32>()
        .map_err(|_| format!("invalid height in resolution '{value}'"))?;

    if width == 0 || height == 0 {
        return Err(format!("resolution '{value}' must be non-zero"));
    }

    Ok((width, height))
}

fn parse_resolution_arg(args: &[String], flag: &str) -> Result<Option<(u32, u32)>, String> {
    let Some(flag_index) = args.iter().position(|arg| arg == flag) else {
        return Ok(None);
    };

    let Some(value) = args.get(flag_index + 1) else {
        return Err(format!("{flag} requires WIDTHxHEIGHT"));
    };

    parse_resolution(value).map(Some)
}

fn parse_usize_arg(args: &[String], flag: &str) -> Result<Option<usize>, String> {
    let Some(flag_index) = args.iter().position(|arg| arg == flag) else {
        return Ok(None);
    };

    let Some(value) = args.get(flag_index + 1) else {
        return Err(format!("{flag} requires a non-negative integer"));
    };

    value
        .parse::<usize>()
        .map(Some)
        .map_err(|_| format!("invalid value '{value}' for {flag}"))
}

fn parse_u32_arg(args: &[String], flag: &str) -> Result<Option<u32>, String> {
    let Some(flag_index) = args.iter().position(|arg| arg == flag) else {
        return Ok(None);
    };

    let Some(value) = args.get(flag_index + 1) else {
        return Err(format!("{flag} requires a non-negative integer"));
    };

    value
        .parse::<u32>()
        .map(Some)
        .map_err(|_| format!("invalid value '{value}' for {flag}"))
}

fn selected_scenes(args: &[String]) -> Result<Vec<Scene>, String> {
    let Some(scene_flag_index) = args.iter().position(|arg| arg == "--scene") else {
        return Ok(SCENE_MATRIX_SCENES.to_vec());
    };

    let Some(scene_name) = args.get(scene_flag_index + 1) else {
        return Err("--scene requires one of: scratch, boundary, period2, all".to_string());
    };

    match scene_name.as_str() {
        "scratch" => Ok(vec![SCRATCH_SCENE]),
        "boundary" => Ok(vec![BOUNDARY_SCENE]),
        "period2" | "interior" => Ok(vec![PERIOD2_SCENE]),
        "all" => Ok(SCENE_MATRIX_SCENES.to_vec()),
        other => Err(format!(
            "unknown scene '{other}'. Valid values: scratch, boundary, period2, all"
        )),
    }
}

/// Build a view centred on the test coordinates at the given zoom.
fn test_view(width: u32, height: u32, zoom: f64) -> FractalView {
    let mut v = FractalView::new(width, height);
    v.center_x = CENTER_X;
    v.center_y = CENTER_Y;
    v.zoom = zoom;
    v
}

fn scene_view(scene: Scene, width: u32, height: u32, zoom: f64) -> FractalView {
    let mut v = FractalView::new(width, height);
    v.center_x = scene.center_x;
    v.center_y = scene.center_y;
    v.zoom = zoom;
    v
}

// --- Main -------------------------------------------------------------------

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let quick = args.contains(&"--quick".to_string());
    let matrix_only = args.contains(&"--matrix-only".to_string());
    let scenes = match selected_scenes(&args) {
        Ok(scenes) => scenes,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(2);
        }
    };
    let warmup_runs_override = match parse_usize_arg(&args, "--warmup-runs") {
        Ok(value) => value,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(2);
        }
    };
    let bench_runs_override = match parse_usize_arg(&args, "--bench-runs") {
        Ok(value) => value,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(2);
        }
    };
    let scene_matrix_iter_override = match parse_u32_arg(&args, "--iterations") {
        Ok(value) => value,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(2);
        }
    };
    let preview_res = match parse_resolution_arg(&args, "--preview-res") {
        Ok(value) => value,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(2);
        }
    };

    // quick mode: fewer runs, tiny grids to finish in ~5 minutes.
    // full mode: 5 runs, larger grids for reliable medians.
    let (mut warmup_runs, mut bench_runs, mut quality_size, mut timing_w, mut timing_h, mut hp_timing_w, mut hp_timing_h) = if quick {
        (1usize, 2usize, 32u32, 160u32, 90u32, 64u32, 36u32)
    } else {
        (2usize, 5usize, 64u32, 320u32, 180u32, 128u32, 72u32)
    };

    if let Some(override_runs) = warmup_runs_override {
        warmup_runs = override_runs;
    }
    if let Some(override_runs) = bench_runs_override {
        bench_runs = override_runs.max(1);
    }

    let scene_matrix_iter = scene_matrix_iter_override.unwrap_or(SCENE_MATRIX_ITER);

    if let Some((preview_w, preview_h)) = preview_res {
        timing_w = preview_w;
        timing_h = preview_h;
        hp_timing_w = (preview_w / 4).max(64);
        hp_timing_h = (preview_h / 4).max(36);
        quality_size = quality_size.max(64);
    }

    let fractal = Mandelbrot::new();
    let params = mandelbrot_params();

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║   Perturbation Theory vs Hi-Precision CPU — Goldilocks Benchmark            ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Test center  : ({:.15}, {:.15})", CENTER_X, CENTER_Y);
    println!("  Timing grid  : {}×{}", timing_w, timing_h);
    println!("  HiPrec grid  : {}×{}", hp_timing_w, hp_timing_h);
    println!("  Quality patch: {}×{}", quality_size, quality_size);
    println!("  Bench runs   : {} (warmup {})", bench_runs, warmup_runs);
    println!("  Max threads  : {}", MAX_THREADS);
    println!("  Mode         : {}", if quick { "QUICK (reduced fidelity)" } else { "FULL" });
    println!("  Scenes       : {}", scenes.iter().map(|scene| scene.key).collect::<Vec<_>>().join(", "));
    if preview_res.is_some() {
        println!("  Override     : preview-resolution matrix pass");
    }
    if warmup_runs_override.is_some() || bench_runs_override.is_some() {
        println!("  Sampling     : explicit run-count override");
    }
    println!();

    let col_w = 100;

    // -----------------------------------------------------------------------
    // SECTION 0: Deep scene matrix — compare CPU f64, HiPrec, and PT across
    // an adversarial scene, a representative boundary scene, and a best-case
    // long-lived interior scene.
    // -----------------------------------------------------------------------
    let scene_matrix_pt_bits = PT_REFERENCE_BITS;

    section(&format!(
        "SECTION 0  Deep Scene Matrix — CPU f64 vs HiPrec-256 vs PT-{}",
        scene_matrix_pt_bits
    ));
    println!(
        "  Zoom: {:.3e}   Iterations: {}   PT rows: 1x1 and 4x4 tiled orbits",
        SCENE_MATRIX_ZOOM,
        scene_matrix_iter,
    );
    println!(
        "  HiPrec timing grid: {}×{} scaled to {}×{}",
        hp_timing_w, hp_timing_h, timing_w, timing_h,
    );
    println!();
    println!(
        "  {:<12}  {:<16}  {:>10}  {:>12}  {:>8}  {}",
        "Scene", "Method", "Time(ms)", "Eff.Time(ms)", "Glitch%", "Notes"
    );
    hr(col_w);

    let scene_hp_pixel_ratio = (timing_w * timing_h) as f64 / (hp_timing_w * hp_timing_h) as f64;

    for scene in scenes {
        let view = scene_view(scene, timing_w, timing_h, SCENE_MATRIX_ZOOM);
        let hp_view = scene_view(scene, hp_timing_w, hp_timing_h, SCENE_MATRIX_ZOOM);

        let cpu_ms = timed_median(
            || { let _ = compute_iterations(&view, scene_matrix_iter, &fractal, &params); },
            warmup_runs, bench_runs,
        );
        println!(
            "  {:<12}  {:<16}  {:>10.2}  {:>12}  {:>8}  {}",
            scene.key,
            "CPU f64",
            cpu_ms,
            format!("{:>12.2}", cpu_ms),
            "  -   ",
            "timing floor only; quality is not measured in this section"
        );

        let hp_ms = timed_median(
            || { let _ = compute_iterations_hiprec(&hp_view, scene_matrix_iter, &fractal, &params, 256, MAX_THREADS); },
            warmup_runs, bench_runs,
        );
        let hp_eff_ms = hp_ms * scene_hp_pixel_ratio;

        for &tiles in SCENE_MATRIX_PT_TILES {
            let t = tiles as usize;
            let orbits = ReferenceOrbit::compute_tile_orbits(&view, tiles, scene_matrix_iter, scene_matrix_pt_bits);
            let pt_ms = timed_median(
                || {
                    let _ = render_perturbation_tiled(
                        &view, &orbits, t, t, &fractal, &params, scene_matrix_iter, scene_matrix_pt_bits, MAX_THREADS, 1.0,
                    );
                },
                warmup_runs, bench_runs,
            );
            let pt_result = render_perturbation_tiled(
                &view, &orbits, t, t, &fractal, &params, scene_matrix_iter, scene_matrix_pt_bits, MAX_THREADS, 1.0,
            );
            let total_px = (view.width * view.height) as f64;
            let glitch_pct = pt_result.glitch_count as f64 / total_px * 100.0;
            let note = if hp_eff_ms > 0.0 {
                speedup(pt_ms, hp_eff_ms)
            } else {
                "    -   ".to_string()
            };

            println!(
                "  {:<12}  {:<16}  {:>10.2}  {:>12.2}  {:>7.2}%  {}",
                "",
                format!("PT-{} {}x{}", scene_matrix_pt_bits, tiles, tiles),
                pt_ms,
                pt_ms,
                glitch_pct,
                note,
            );
        }

        println!(
            "  {:<12}  {:<16}  {:>10.2}  {:>12.2}  {:>8}  {}",
            "",
            "HiPrec-256",
            hp_ms,
            hp_eff_ms,
            "  -   ",
            format!("{}×{} scaled | {}", hp_timing_w, hp_timing_h, scene.notes),
        );
        println!();
    }

    if matrix_only {
        return;
    }

    // -----------------------------------------------------------------------
    // SECTION 1: Coarse zoom sweep — PT-256b vs CPU f64 only (no HiPrec)
    // Purpose: quickly find where PT starts winning.
    // Uses a low iteration count so shallow-zoom rows complete fast.
    // -----------------------------------------------------------------------
    section("SECTION 1  Coarse Zoom Sweep — PT-256b vs CPU f64  (1024 iter, no HiPrec)");
    println!("  (HiPrec excluded here — see Section 2 for the crossover comparison)");
    println!();
    println!(
        "  {:<14}  {:<18}  {:>10}  {:>8}  {}",
        "Zoom", "Method", "Time(ms)", "Glitch%", "Notes"
    );
    hr(col_w);

    let coarse_iter = 1024u32;

    for &zoom in COARSE_ZOOM_SWEEP {
        let view = test_view(timing_w, timing_h, zoom);

        // CPU f64 baseline
        let f64_ms = timed_median(
            || { let _ = compute_iterations(&view, coarse_iter, &fractal, &params); },
            warmup_runs, bench_runs,
        );

        let f64_note = if zoom >= PT_LOW_ZOOM_THRESHOLD * 10.0 {
            "WARNING: f64 precision broken"
        } else if zoom >= PT_LOW_ZOOM_THRESHOLD {
            "f64 marginal"
        } else {
            "f64 sufficient"
        };

        println!(
            "  {:<14.3e}  {:<18}  {:>10.2}  {:>8}  {}",
            zoom, "CPU f64", f64_ms, "  -   ", f64_note
        );

        // PT with 256-bit reference orbit
        let orbit = ReferenceOrbit::compute_mandelbrot(CENTER_X, CENTER_Y, zoom, coarse_iter, 256);
        let pt_ms = timed_median(
            || { let _ = render_perturbation(&view, &orbit, &fractal, &params, coarse_iter, 256, MAX_THREADS, 1.0); },
            warmup_runs, bench_runs,
        );
        let pt_result = render_perturbation(&view, &orbit, &fractal, &params, coarse_iter, 256, MAX_THREADS, 1.0);
        let total_px = (view.width * view.height) as f64;
        let glitch_pct = pt_result.glitch_count as f64 / total_px * 100.0;
        let pt_warn = if pt_result.low_zoom_warning { "  WARNING: below PT threshold" } else { "" };

        println!(
            "  {:<14}  {:<18}  {:>10.2}  {:>8}  {}{}",
            "", "PT(ref=256b)", pt_ms,
            format!("{:5.1}%", glitch_pct),
            speedup(pt_ms, f64_ms),
            pt_warn,
        );
        println!();
    }

    // -----------------------------------------------------------------------
    // SECTION 2: PT vs HiPrec-128 crossover detail (1e12–1e14)
    // Uses a smaller HiPrec grid to keep runtime manageable.
    // -----------------------------------------------------------------------
    section("SECTION 2  PT vs HiPrec-128 Crossover Detail  (1024 iter, finds the tipping point)");
    println!(
        "  HiPrec timing grid: {}×{} (scaled to equivalent {}×{})",
        hp_timing_w, hp_timing_h, timing_w, timing_h
    );
    println!();
    println!(
        "  {:<14}  {:<18}  {:>12}  {:>12}  {:>10}  {}",
        "Zoom", "Method", "Time(ms)", "Eff.Time(ms)", "Glitch%", "Verdict"
    );
    hr(col_w);

    let crossover_iter = 1024u32;
    let hp_pixel_ratio = (timing_w * timing_h) as f64 / (hp_timing_w * hp_timing_h) as f64;

    for &zoom in FINE_ZOOM_CROSSOVER {
        let view = test_view(timing_w, timing_h, zoom);
        let hp_view = test_view(hp_timing_w, hp_timing_h, zoom);

        // PT
        let orbit = ReferenceOrbit::compute_mandelbrot(CENTER_X, CENTER_Y, zoom, crossover_iter, 256);
        let pt_ms = timed_median(
            || { let _ = render_perturbation(&view, &orbit, &fractal, &params, crossover_iter, 256, MAX_THREADS, 1.0); },
            warmup_runs, bench_runs,
        );
        let pt_result = render_perturbation(&view, &orbit, &fractal, &params, crossover_iter, 256, MAX_THREADS, 1.0);
        let glitch_pct = pt_result.glitch_count as f64 / (view.width * view.height) as f64 * 100.0;

        // HiPrec-128 (smaller grid, scale timing)
        let hp_ms = timed_median(
            || { let _ = compute_iterations_hiprec(&hp_view, crossover_iter, &fractal, &params, 128, MAX_THREADS); },
            warmup_runs, bench_runs,
        );
        let hp_effective_ms = hp_ms * hp_pixel_ratio;

        let verdict = if pt_ms < hp_effective_ms {
            format!("PT wins by {:.1}x", hp_effective_ms / pt_ms)
        } else {
            format!("HiPrec wins by {:.1}x", pt_ms / hp_effective_ms)
        };

        println!(
            "  {:<14.3e}  {:<18}  {:>12.2}  {:>12.2}  {:>10.1}%  {}",
            zoom, "PT(ref=256b)", pt_ms, pt_ms, glitch_pct, verdict
        );
        println!(
            "  {:<14}  {:<18}  {:>12.2}  {:>12.2}  {:>10}  ({}×{} scaled)",
            "", "HiPrec-128", hp_ms, hp_effective_ms, "  -   ",
            hp_timing_w, hp_timing_h
        );
        println!();
    }

    // -----------------------------------------------------------------------
    // SECTION 3: Bit-Width Comparison at Deep Zoom Test Coords
    // Both accuracy (quality patch) and timing (full grid).
    // -----------------------------------------------------------------------
    section("SECTION 3  Bit-Width Comparison at Deep Zoom (zoom=1.55e13, 4096 iter)");

    let deep_zoom = 1.55e13_f64;
    let deep_iter = 4096u32;
    let deep_view = test_view(timing_w, timing_h, deep_zoom);

    // Quality patch — small grid, HiPrec-1024 is ground truth
    let q_view = test_view(quality_size, quality_size, deep_zoom);

    println!("  Computing HiPrec-1024 ground truth ({}×{})…", quality_size, quality_size);
    let gt = ground_truth(&q_view, deep_iter, &fractal);
    println!("  Ground truth computed ({} pixels).", gt.len());
    println!();
    println!(
        "  {:<18}  {:>10}  {:>8}  {:>14}  {:>14}  {}",
        "Method", "Time(ms)", "Glitch%", "Match vs GT", "Orbit(ms)", "Notes"
    );
    hr(col_w);

    // CPU f64 (will be broken at this zoom)
    {
        let f64_t_ms = timed_median(
            || { let _ = compute_iterations(&deep_view, deep_iter, &fractal, &params); },
            warmup_runs, bench_runs,
        );
        let f64_q = compute_iterations(&q_view, deep_iter, &fractal, &params);
        let (diff, tot) = pixel_diff(&f64_q, &gt);
        println!(
            "  {:<18}  {:>10.2}  {:>8}  {:>14}  {:>14}  BROKEN — f64 unusable at 1.55e13",
            "CPU f64", f64_t_ms, "  -   ",
            format!("{}/{} bad", diff, tot), "n/a",
        );
    }

    println!();

    // PT at each orbit bit width
    for &pt_bits in PT_ORBIT_BITS {
        let t_orbit = Instant::now();
        let orbit = ReferenceOrbit::compute_mandelbrot(CENTER_X, CENTER_Y, deep_zoom, deep_iter, pt_bits);
        let orbit_ms = t_orbit.elapsed().as_secs_f64() * 1000.0;

        let pt_t_ms = timed_median(
            || { let _ = render_perturbation(&deep_view, &orbit, &fractal, &params, deep_iter, pt_bits, MAX_THREADS, 1.0); },
            warmup_runs, bench_runs,
        );

        let pt_q_result = render_perturbation(&q_view, &orbit, &fractal, &params, deep_iter, pt_bits, MAX_THREADS, 1.0);
        let (diff, tot) = pixel_diff(&pt_q_result.iterations, &gt);
        let glitch_pct = pt_q_result.glitch_count as f64 / tot as f64 * 100.0;
        let match_pct = 100.0 - diff as f64 / tot as f64 * 100.0;

        println!(
            "  {:<18}  {:>10.2}  {:>8}  {:>14}  {:>14}  orbit cached across frames",
            format!("PT(ref={}b)", pt_bits),
            pt_t_ms,
            format!("{:5.1}%", glitch_pct),
            format!("{:.1}% match", match_pct),
            format!("{:.1}ms", orbit_ms),
        );
    }

    println!();

    // HiPrec at each bit width (smaller grid for 256+, scale timing)
    for &bits in BIT_WIDTHS {
        let (tw, th, note) = if bits >= 512 {
            let tw = hp_timing_w / 2;
            let th = hp_timing_h / 2;
            (tw, th, format!("scaled from {}×{}", tw, th))
        } else if bits >= 256 {
            (hp_timing_w, hp_timing_h, format!("scaled from {}×{}", hp_timing_w, hp_timing_h))
        } else {
            (timing_w, timing_h, "full grid".to_string())
        };

        let tv = test_view(tw, th, deep_zoom);
        let hp_ms = timed_median(
            || { let _ = compute_iterations_hiprec(&tv, deep_iter, &fractal, &params, bits, MAX_THREADS); },
            warmup_runs, bench_runs,
        );
        let pixel_scale = (timing_w * timing_h) as f64 / (tw * th) as f64;
        let effective_ms = hp_ms * pixel_scale;

        let hp_q = compute_iterations_hiprec(&q_view, deep_iter, &fractal, &params, bits, MAX_THREADS).unwrap();
        let (diff, tot) = pixel_diff(&hp_q, &gt);
        let match_pct = 100.0 - diff as f64 / tot as f64 * 100.0;

        println!(
            "  {:<18}  {:>10.2}  {:>8}  {:>14}  {:>14}  {}",
            format!("HiPrec-{}b", bits),
            effective_ms,
            "  -   ",
            format!("{:.1}% match", match_pct),
            "n/a",
            note,
        );
    }

    // -----------------------------------------------------------------------
    // SECTION 4: Iteration Count Scaling (deep zoom, PT-256b vs HiPrec-128)
    // -----------------------------------------------------------------------
    section("SECTION 4  Iteration Count Scaling  (zoom=1.55e13, PT-256b vs HiPrec-128)");
    println!(
        "  HiPrec timing at {}×{}, scaled to {}×{}",
        hp_timing_w, hp_timing_h, timing_w, timing_h
    );
    println!();
    println!(
        "  {:<8}  {:>14}  {:>16}  {:>16}  {:>10}",
        "MaxIter", "PT-256b(ms)", "HiPrec-128(eff.ms)", "Speedup(PT)", "PT Glitch%"
    );
    hr(col_w);

    let mut prev_glitch_pct = 0.0_f64;
    'iter_sweep: for &iter_count in ITER_SWEEP {
        // If the previous row already had near-100% glitch, continuing would just run
        // a near-full HiPrec render for every iteration count — no new information.
        if prev_glitch_pct > 90.0 {
            println!(
                "  {:>8}  (skipped — previous row had {:.1}% glitch, remaining rows degenerate)",
                iter_count, prev_glitch_pct
            );
            continue 'iter_sweep;
        }

        let orbit = ReferenceOrbit::compute_mandelbrot(CENTER_X, CENTER_Y, deep_zoom, iter_count, 256);
        let pt_view = test_view(timing_w, timing_h, deep_zoom);
        let hp_view2 = test_view(hp_timing_w, hp_timing_h, deep_zoom);

        let pt_ms = timed_median(
            || { let _ = render_perturbation(&pt_view, &orbit, &fractal, &params, iter_count, 256, MAX_THREADS, 1.0); },
            warmup_runs, bench_runs,
        );
        let pt_result = render_perturbation(&pt_view, &orbit, &fractal, &params, iter_count, 256, MAX_THREADS, 1.0);
        let glitch_pct = pt_result.glitch_count as f64 / (pt_view.width * pt_view.height) as f64 * 100.0;

        let hp_ms = timed_median(
            || { let _ = compute_iterations_hiprec(&hp_view2, iter_count, &fractal, &params, 128, MAX_THREADS); },
            warmup_runs, bench_runs,
        );
        let hp_eff = hp_ms * hp_pixel_ratio;

        println!(
            "  {:>8}  {:>14.2}  {:>16.2}  {:>16}  {:>10.2}%",
            iter_count, pt_ms, hp_eff,
            speedup(pt_ms, hp_eff),
            glitch_pct,
        );

        prev_glitch_pct = glitch_pct;
    }

    // -----------------------------------------------------------------------
    // SECTION 5: PT orbit bit-width vs quality at the debug scene
    // Uses zoom=1.55e13.  Three sub-sections:
    //
    // Part A: HiPrec bit-width check (how much BigFloat precision does this scene need?)
    // Part B: PT correctness validation at 32768 iter, orbit at VIEW CENTER.
    //         After the glitch-fallback fix (screen_to_complex_hiprec in fallback), glitch
    //         pixels should match HiPrec GT exactly.  Orbit depth ~4000 means ~99% glitch,
    //         but correctness should be near-100% post-fix.
    // Part C: PT orbit bit-width comparison at 4096 iter, orbit at VIEW CENTER.
    //         At 4096 iter the orbit serves ~97% of pixels (low glitch), making this the
    //         valid window for comparing 64b vs 512b orbit precision.
    //
    // CRITICAL: orbit center must equal view center.  PT computes dc = c_pixel - view_center
    // internally; using a different orbit center shifts all dc values, corrupting results.
    //
    // This section is intentionally slow (one-time experiment).
    // -----------------------------------------------------------------------
    let s5_iter = 32768u32;
    let s5_w = 32u32;
    let s5_h = 18u32;
    let s5_pixels = (s5_w * s5_h) as usize;
    let s5_view = test_view(s5_w, s5_h, deep_zoom);

    section(&format!(
        "SECTION 5  PT Orbit Bit-Width vs Quality  (zoom=1.55e13, {}x{} patch)",
        s5_w, s5_h
    ));
    println!("  Orbit center is the view center ({:.15}, {:.15}).", CENTER_X, CENTER_Y);
    println!("  This is required: render_perturbation computes dc from the view center,");
    println!("  so orbit center must equal view center for correct results.");
    println!();

    println!("  Computing HiPrec-256b ground truth ({}x{}, {} iter)…", s5_w, s5_h, s5_iter);
    let t_gt5 = Instant::now();
    let gt_s5 = compute_iterations_hiprec(&s5_view, s5_iter, &fractal, &params, 256, MAX_THREADS)
        .expect("HiPrec-256b must succeed");
    let gt5_ms = t_gt5.elapsed().as_secs_f64() * 1000.0;
    println!("  Ground truth: {:.0}ms  ({} pixels)", gt5_ms, gt_s5.len());

    let interior5 = gt_s5.iter().filter(|&&v| v == s5_iter).count();
    let exterior5 = s5_pixels - interior5;
    println!(
        "  Scene mix at {} iter: {} interior, {} boundary/exterior ({:.0}% varied)",
        s5_iter, interior5, exterior5, exterior5 as f64 / s5_pixels as f64 * 100.0
    );
    println!();

    // Orbit depth at view center — tells us when PT degenerates at this location
    let center_orbit_depth = {
        let probe = ReferenceOrbit::compute_mandelbrot(CENTER_X, CENTER_Y, deep_zoom, s5_iter, 256);
        probe.orbit.len()
    };
    println!(
        "  View center orbit depth: {} of {} iter  ({})",
        center_orbit_depth,
        s5_iter,
        if center_orbit_depth as u32 >= s5_iter {
            "interior — orbit never escapes"
        } else {
            "boundary — orbit exhausts before max_iter"
        }
    );
    println!();

    // ---- Part A: HiPrec bit-width check ----
    println!("  --- Part A: HiPrec bit-width check (zoom=1.55e13, {} iter) ---", s5_iter);
    println!(
        "  {:<12}  {:>10}  {:>14}  {}",
        "HiPrec Bits", "Time(ms)", "Bad Pixels", "Assessment"
    );
    hr(col_w);
    for &bits in &[64u32, 128u32] {
        let t = Instant::now();
        let result = compute_iterations_hiprec(&s5_view, s5_iter, &fractal, &params, bits, MAX_THREADS)
            .expect("HiPrec must succeed");
        let elapsed = t.elapsed().as_secs_f64() * 1000.0;
        let (diff, _) = pixel_diff(&result, &gt_s5);
        let bad_pct = diff as f64 / s5_pixels as f64 * 100.0;
        let assessment = if diff == 0 {
            "exact match — sufficient"
        } else if bad_pct < 1.0 {
            "< 1% error — likely acceptable"
        } else if bad_pct < 5.0 {
            "marginal — consider higher bits"
        } else {
            "DEGRADED — use higher bits"
        };
        println!(
            "  {:<12}  {:>10.1}  {:>5}/{} ({:.1}%)  {}",
            format!("{}b", bits), elapsed, diff, s5_pixels, bad_pct, assessment
        );
    }
    println!(
        "  {:<12}  {:>10.1}  {:>5}/{} ({:.1}%)  REFERENCE (ground truth)",
        "256b", gt5_ms, 0, s5_pixels, 0.0
    );
    println!();

    // ---- Part B: PT correctness validation at s5_iter ----
    // Orbit depth < max_iter → near-100% glitch.  After the fallback precision fix,
    // glitch pixels fall back to screen_to_complex_hiprec() + iterate_hiprec(), which
    // is identical to what the ground truth computes.  Match vs GT should be ~100%.
    println!("  --- Part B: PT correctness validation ({} iter, orbit at view center) ---", s5_iter);
    println!("  Orbit exhausts at ~{} iter → expect ~{}% glitch.", center_orbit_depth,
        (1.0 - center_orbit_depth as f64 / s5_iter as f64) * 100.0);
    println!("  Post fallback-fix: glitch pixels use screen_to_complex_hiprec + iterate_hiprec,");
    println!("  identical to ground truth.  Match vs GT should be near 100%.");
    println!("  All rows use HiPrec-256b for glitch fallback.");
    println!(
        "  {:<16}  {:>14}  {:>10}  {:>14}  {:>14}",
        "Orbit Bits", "Orbit Time(ms)", "Glitch%", "Match vs GT", "vs PT-1024"
    );
    hr(col_w);

    let pt1024_orbit_b = ReferenceOrbit::compute_mandelbrot(CENTER_X, CENTER_Y, deep_zoom, s5_iter, 1024);
    let pt1024_result_b = render_perturbation(&s5_view, &pt1024_orbit_b, &fractal, &params, s5_iter, 256, MAX_THREADS, 1.0);

    for &orbit_bits in PT_ORBIT_BITS.iter().chain(&[1024u32]) {
        let t_orbit = Instant::now();
        let orbit = ReferenceOrbit::compute_mandelbrot(CENTER_X, CENTER_Y, deep_zoom, s5_iter, orbit_bits);
        let orbit_ms = t_orbit.elapsed().as_secs_f64() * 1000.0;

        let result = render_perturbation(&s5_view, &orbit, &fractal, &params, s5_iter, 256, MAX_THREADS, 1.0);

        let (diff_gt, tot) = pixel_diff(&result.iterations, &gt_s5);
        let (diff_pt1024, _) = pixel_diff(&result.iterations, &pt1024_result_b.iterations);
        let glitch_pct = result.glitch_count as f64 / tot as f64 * 100.0;
        let match_gt = 100.0 - diff_gt as f64 / tot as f64 * 100.0;
        let match_pt1024 = 100.0 - diff_pt1024 as f64 / tot as f64 * 100.0;

        println!(
            "  {:<16}  {:>14.2}  {:>10.2}%  {:>14}  {:>14}",
            orbit_bits, orbit_ms, glitch_pct,
            format!("{:.1}% match", match_gt),
            if orbit_bits == 1024 { "REFERENCE".to_string() } else { format!("{:.1}% match", match_pt1024) },
        );
    }
    println!();

    // ---- Part C: PT orbit bit-width comparison at 4096 iter ----
    // Orbit depth (~4000) is within max_iter → PT serves ~97% of pixels (low glitch).
    // This is the meaningful comparison for orbit bit-width quality on this scene.
    let s5c_iter = 4096u32;
    let gt_s5c = compute_iterations_hiprec(&s5_view, s5c_iter, &fractal, &params, 256, MAX_THREADS)
        .expect("HiPrec must succeed");
    let interior5c = gt_s5c.iter().filter(|&&v| v == s5c_iter).count();
    let exterior5c = s5_pixels - interior5c;

    println!("  --- Part C: PT orbit bit-width comparison ({} iter, orbit at view center) ---", s5c_iter);
    println!(
        "  Scene mix at {} iter: {} interior, {} exterior ({:.0}% varied)",
        s5c_iter, interior5c, exterior5c, exterior5c as f64 / s5_pixels as f64 * 100.0
    );
    println!("  All rows use HiPrec-256b for glitch fallback.");
    println!(
        "  {:<16}  {:>14}  {:>10}  {:>14}  {:>14}",
        "Orbit Bits", "Orbit Time(ms)", "Glitch%", "Match vs GT", "vs PT-1024"
    );
    hr(col_w);

    let pt1024_orbit_c = ReferenceOrbit::compute_mandelbrot(CENTER_X, CENTER_Y, deep_zoom, s5c_iter, 1024);
    let pt1024_result_c = render_perturbation(&s5_view, &pt1024_orbit_c, &fractal, &params, s5c_iter, 256, MAX_THREADS, 1.0);

    for &orbit_bits in PT_ORBIT_BITS.iter().chain(&[1024u32]) {
        let t_orbit = Instant::now();
        let orbit = ReferenceOrbit::compute_mandelbrot(CENTER_X, CENTER_Y, deep_zoom, s5c_iter, orbit_bits);
        let orbit_ms = t_orbit.elapsed().as_secs_f64() * 1000.0;

        let result = render_perturbation(&s5_view, &orbit, &fractal, &params, s5c_iter, 256, MAX_THREADS, 1.0);

        let (diff_gt, tot) = pixel_diff(&result.iterations, &gt_s5c);
        let (diff_pt1024, _) = pixel_diff(&result.iterations, &pt1024_result_c.iterations);
        let glitch_pct = result.glitch_count as f64 / tot as f64 * 100.0;
        let match_gt = 100.0 - diff_gt as f64 / tot as f64 * 100.0;
        let match_pt1024 = 100.0 - diff_pt1024 as f64 / tot as f64 * 100.0;

        println!(
            "  {:<16}  {:>14.2}  {:>10.2}%  {:>14}  {:>14}",
            orbit_bits, orbit_ms, glitch_pct,
            format!("{:.1}% match", match_gt),
            if orbit_bits == 1024 { "REFERENCE".to_string() } else { format!("{:.1}% match", match_pt1024) },
        );
    }

    // -----------------------------------------------------------------------
    // SECTION 6: f64 precision breakdown
    // -----------------------------------------------------------------------
    section("SECTION 6  f64 Precision Breakdown  (4096 iter, 64×64 patch, vs HiPrec-256)");
    println!(
        "  {:<14}  {:>10}  {:>14}  {}",
        "Zoom", "f64 Time", "Bad Pixels", "Assessment"
    );
    hr(col_w);

    let fine_zooms: &[f64] = &[
        1.0e5, 1.0e7, 1.0e9, 5.0e9,
        1.0e10, 5.0e10, 1.0e11, 5.0e11,
        1.0e12, 5.0e12, 1.55e13,
    ];
    let f64_check_iter = 4096u32;

    for &zoom in fine_zooms {
        let sv = test_view(64, 64, zoom);

        let f64_t = timed_median(
            || { let _ = compute_iterations(&sv, f64_check_iter, &fractal, &params); },
            1, 2,
        );
        let f64_i = compute_iterations(&sv, f64_check_iter, &fractal, &params);
        let hp_i = compute_iterations_hiprec(&sv, f64_check_iter, &fractal, &params, 256, MAX_THREADS).unwrap();
        let (diff, tot) = pixel_diff(&f64_i, &hp_i);
        let bad_pct = diff as f64 / tot as f64 * 100.0;

        let assessment = if bad_pct < 0.1 {
            "ACCURATE   — f64 sufficient"
        } else if bad_pct < 5.0 {
            "SLIGHT DRIFT — marginal"
        } else if bad_pct < 30.0 {
            "NOTICEABLE ERRORS — switch recommended"
        } else {
            "BROKEN     — f64 unusable at this zoom"
        };

        println!(
            "  {:<14.3e}  {:>10.2}  {:>14}  {}",
            zoom, f64_t,
            format!("{}/{} ({:.1}%)", diff, tot, bad_pct),
            assessment,
        );
    }

    // -----------------------------------------------------------------------
    // GOLDILOCKS SUMMARY
    // -----------------------------------------------------------------------
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║   GOLDILOCKS SUMMARY                                                        ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Zoom Range         Recommended Method      Reason");
    hr(col_w);
    println!("  < 1e10             CPU f64                 f64 precise; any other method is 100-500x SLOWER");
    println!("  1e10 – 1e12        CPU f64 (marginally)    f64 drifts but PT glitch rate is 95-100% here");
    println!("                     HiPrec-64               Better choice: cheap, covers drift");
    println!("  1e12 – 5e12        PT crossover zone       PT glitch rate drops from ~95% toward low single digits");
    println!("                                              Run Section 2 to find exact crossover on your hardware");
    println!("  5e12 – 1e15        PT(256b ref)             SWEET SPOT: glitch < 5%, orbit amortizes across frames");
    println!("                                              Typical speedup: 5-20x over HiPrec-128");
    println!("  > 1e15             PT(512b ref)             Higher orbit bits reduce remaining glitches");
    println!();
    println!("  KEY INSIGHT: PT glitch rate is strongly dependent on the reference point.");
    println!("  At very deep zoom the pixel offsets (dc) are microscopically small,");
    println!("  keeping |dz| << escape_radius for the entire orbit → near-zero glitches.");
    println!("  At shallow zoom, large dc values cause dz to grow past the escape radius");
    println!("  for many pixels, forcing expensive hi-prec fallbacks on almost every pixel.");
    println!();
    println!("  Orbit caching: the BigFloat orbit is computed ONCE per view pan/zoom.");
    println!("  Its cost (see Section 3 orbit_ms column) amortizes to near-zero in");
    println!("  interactive use where the same orbit is reused across many preview frames.");
    println!();
}
