//! Deep scene benchmark for CPU f64 vs CPU Hi-Prec vs Perturbation Theory.
//!
//! This is a focused scene-matrix benchmark built from the existing
//! `perturbation_bench.rs` timing/quality approach plus the scene selection work
//! from `pt_tiling_test.rs`.
//!
//! It compares three workload classes:
//! - `scratch`  : adversarial exterior filament (PT expected to struggle)
//! - `boundary` : representative long-dwell boundary point
//! - `period2`  : best-case long-lived interior point
//!
//! Run:
//!   cargo run --release -p forma-fractalis-core --example pt_mode_benchmark
//!
//! Quick run:
//!   cargo run --release -p forma-fractalis-core --example pt_mode_benchmark -- --quick
//!
//! Select scene(s):
//!   cargo run --release -p forma-fractalis-core --example pt_mode_benchmark -- --scene scratch
//!   cargo run --release -p forma-fractalis-core --example pt_mode_benchmark -- --scene boundary
//!   cargo run --release -p forma-fractalis-core --example pt_mode_benchmark -- --scene period2
//!   cargo run --release -p forma-fractalis-core --example pt_mode_benchmark -- --scene all

use forma_fractalis_core::{
    fractals::{FractalView, Mandelbrot},
    perturbation::{render_perturbation_tiled, ReferenceOrbit},
    rendering::{compute_iterations, compute_iterations_hiprec},
};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone, Copy)]
struct Scene {
    key: &'static str,
    title: &'static str,
    center_x: f64,
    center_y: f64,
    zoom: f64,
    pt_tiles: u32,
    notes: &'static str,
}

const SCRATCH_SCENE: Scene = Scene {
    key: "scratch",
    title: "Scratch exterior filament",
    center_x: -1.2541275710056559,
    center_y: 0.38365671509396854,
    zoom: 1.55e13,
    pt_tiles: 8,
    notes: "Deep exterior filament scene. At modest iteration counts, aggressive tiling can still make PT look strong; at higher counts this becomes a harsher stress case.",
};

const BOUNDARY_SCENE: Scene = Scene {
    key: "boundary",
    title: "Seahorse boundary",
    center_x: -0.743643887037151,
    center_y: 0.131825904205330,
    zoom: 1.0e11,
    pt_tiles: 8,
    notes: "Representative long-dwell boundary scene. Useful midpoint between the exterior filament stress case and the easy interior case.",
};

const PERIOD2_SCENE: Scene = Scene {
    key: "period2",
    title: "Period-2 bulb interior",
    center_x: -1.0,
    center_y: 0.0,
    zoom: 1.55e13,
    pt_tiles: 1,
    notes: "Best-case single-reference scene. One long-lived orbit should let PT avoid almost all fallback.",
};

const ALL_SCENES: [Scene; 3] = [SCRATCH_SCENE, BOUNDARY_SCENE, PERIOD2_SCENE];

const MAX_THREADS: usize = 18;
const MAX_ITER: u32 = 4096;
const PT_BITS: u32 = 256;
const PT_FALLBACK_BITS: u32 = 256;
const HIPREC_BITS: u32 = 128;
const GT_BITS: u32 = 256;

fn mandelbrot_params() -> HashMap<String, f64> {
    let mut p = HashMap::new();
    p.insert("power".to_string(), 2.0);
    p.insert("escape_radius".to_string(), 2.0);
    p
}

fn selected_scenes(args: &[String]) -> Result<Vec<Scene>, String> {
    let Some(scene_flag_index) = args.iter().position(|arg| arg == "--scene") else {
        return Ok(ALL_SCENES.to_vec());
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

fn make_view(scene: Scene, width: u32, height: u32) -> FractalView {
    let mut view = FractalView::new(width, height);
    view.center_x = scene.center_x;
    view.center_y = scene.center_y;
    view.zoom = scene.zoom;
    view
}

fn timed_median<F: FnMut()>(mut f: F, warmup: usize, runs: usize) -> f64 {
    for _ in 0..warmup {
        f();
    }
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

fn pixel_diff(a: &[u32], b: &[u32]) -> (usize, usize) {
    let mismatches = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
    (mismatches, a.len())
}

fn hr(width: usize) {
    println!("{}", "─".repeat(width));
}

fn section(title: &str) {
    println!();
    println!("╔  {}  {}", title, "─".repeat(78usize.saturating_sub(title.len() + 4)));
    println!();
}

fn speedup_label(method_ms: f64, baseline_ms: f64) -> String {
    if baseline_ms <= 0.0 {
        return "n/a".to_string();
    }
    let ratio = baseline_ms / method_ms;
    if ratio >= 1.0 {
        format!("{:5.2}x faster", ratio)
    } else {
        format!("{:5.2}x slower", 1.0 / ratio)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let quick = args.contains(&"--quick".to_string());
    let scenes = match selected_scenes(&args) {
        Ok(scenes) => scenes,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(2);
        }
    };

    let (warmup_runs, bench_runs, timing_w, timing_h, hp_timing_w, hp_timing_h, quality_w, quality_h) = if quick {
        (1usize, 2usize, 160u32, 90u32, 64u32, 36u32, 24u32, 24u32)
    } else {
        (2usize, 4usize, 320u32, 180u32, 128u32, 72u32, 40u32, 40u32)
    };

    let hp_pixel_ratio = (timing_w * timing_h) as f64 / (hp_timing_w * hp_timing_h) as f64;
    let fractal = Mandelbrot::new();
    let params = mandelbrot_params();

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║   Deep Scene Benchmark — CPU f64 vs Hi-Prec vs Perturbation                 ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Scenes       : {}", scenes.iter().map(|s| s.key).collect::<Vec<_>>().join(", "));
    println!("  Max iter     : {}", MAX_ITER);
    println!("  PT bits      : orbit {} / fallback {}", PT_BITS, PT_FALLBACK_BITS);
    println!("  HiPrec bits  : timing {} / ground truth {}", HIPREC_BITS, GT_BITS);
    println!("  Timing grid  : {}x{}", timing_w, timing_h);
    println!("  HiPrec grid  : {}x{} (scaled to timing grid)", hp_timing_w, hp_timing_h);
    println!("  Quality patch: {}x{}", quality_w, quality_h);
    println!("  Bench runs   : {} (warmup {})", bench_runs, warmup_runs);
    println!("  Max threads  : {}", MAX_THREADS);
    println!("  Mode         : {}", if quick { "QUICK" } else { "FULL" });

    section("SECTION 1  Deep Scene Matrix");
    println!("  Measures CPU f64, HiPrec-128, and scene-tuned tiled PT at deep zoom.");
    println!("  HiPrec timing is measured on a smaller grid and scaled to the timing grid.");
    println!("  Quality is measured against a HiPrec-256 patch for each scene.");

    for scene in scenes {
        let view = make_view(scene, timing_w, timing_h);
        let hp_view = make_view(scene, hp_timing_w, hp_timing_h);
        let q_view = make_view(scene, quality_w, quality_h);

        println!();
        println!("  Scene: {} [{}]", scene.title, scene.key);
        println!("  Center: ({:.15}, {:.15})  zoom={:.3e}", scene.center_x, scene.center_y, scene.zoom);
        println!("  PT tiles: {}x{}", scene.pt_tiles, scene.pt_tiles);
        println!("  Note: {}", scene.notes);
        println!();
        println!(
            "  {:<18}  {:>10}  {:>10}  {:>10}  {:>10}  {:>12}  {}",
            "Method", "Orbit(ms)", "Render(ms)", "Total(ms)", "Glitch%", "Match vs GT", "Notes"
        );
        hr(112);

        let gt = compute_iterations_hiprec(&q_view, MAX_ITER, &fractal, &params, GT_BITS, MAX_THREADS)
            .expect("HiPrec ground truth must succeed");

        let cpu_ms = timed_median(
            || {
                let _ = compute_iterations(&view, MAX_ITER, &fractal, &params);
            },
            warmup_runs,
            bench_runs,
        );
        let cpu_q = compute_iterations(&q_view, MAX_ITER, &fractal, &params);
        let (cpu_bad, cpu_total) = pixel_diff(&cpu_q, &gt);
        let cpu_match = 100.0 - cpu_bad as f64 / cpu_total as f64 * 100.0;
        let cpu_note = if cpu_match < 100.0 {
            "mismatch vs HiPrec patch"
        } else {
            "matched this patch"
        };
        println!(
            "  {:<18}  {:>10}  {:>10}  {:>10.2}  {:>10}  {:>11.1}%  {}",
            "CPU f64",
            "n/a",
            "n/a",
            cpu_ms,
            "n/a",
            cpu_match,
            cpu_note,
        );

        let hp_ms = timed_median(
            || {
                let _ = compute_iterations_hiprec(&hp_view, MAX_ITER, &fractal, &params, HIPREC_BITS, MAX_THREADS);
            },
            warmup_runs,
            bench_runs,
        );
        let hp_effective_ms = hp_ms * hp_pixel_ratio;
        let hp_q = compute_iterations_hiprec(&q_view, MAX_ITER, &fractal, &params, HIPREC_BITS, MAX_THREADS)
            .expect("HiPrec quality patch must succeed");
        let (hp_bad, hp_total) = pixel_diff(&hp_q, &gt);
        let hp_match = 100.0 - hp_bad as f64 / hp_total as f64 * 100.0;
        println!(
            "  {:<18}  {:>10}  {:>10}  {:>10.2}  {:>10}  {:>11.1}%  scaled from {}x{}",
            format!("HiPrec-{}b", HIPREC_BITS),
            "n/a",
            "n/a",
            hp_effective_ms,
            "n/a",
            hp_match,
            hp_timing_w,
            hp_timing_h,
        );

        let pt_tiles = scene.pt_tiles as usize;
        let pt_orbit_ms = timed_median(
            || {
                let _ = ReferenceOrbit::compute_tile_orbits(&view, scene.pt_tiles, MAX_ITER, PT_BITS);
            },
            warmup_runs,
            bench_runs,
        );
        let orbits = ReferenceOrbit::compute_tile_orbits(&view, scene.pt_tiles, MAX_ITER, PT_BITS);
        let orbits_ref = &orbits;
        let pt_render_ms = timed_median(
            || {
                let _ = render_perturbation_tiled(
                    &view,
                    orbits_ref,
                    pt_tiles,
                    pt_tiles,
                    &fractal,
                    &params,
                    MAX_ITER,
                    PT_FALLBACK_BITS,
                    MAX_THREADS,
                    1.0,
                );
            },
            warmup_runs,
            bench_runs,
        );
        let pt_total_ms = pt_orbit_ms + pt_render_ms;

        let q_orbits = ReferenceOrbit::compute_tile_orbits(&q_view, scene.pt_tiles, MAX_ITER, PT_BITS);
        let pt_q = render_perturbation_tiled(
            &q_view,
            &q_orbits,
            pt_tiles,
            pt_tiles,
            &fractal,
            &params,
            MAX_ITER,
            PT_FALLBACK_BITS,
            MAX_THREADS,
            1.0,
        );
        let (pt_bad, pt_total) = pixel_diff(&pt_q.iterations, &gt);
        let pt_match = 100.0 - pt_bad as f64 / pt_total as f64 * 100.0;
        let pt_glitch = pt_q.glitch_count as f64 / pt_total as f64 * 100.0;
        println!(
            "  {:<18}  {:>10.2}  {:>10.2}  {:>10.2}  {:>9.2}%  {:>11.1}%  {}",
            format!("PT-{}b {}x{}", PT_BITS, scene.pt_tiles, scene.pt_tiles),
            pt_orbit_ms,
            pt_render_ms,
            pt_total_ms,
            pt_glitch,
            pt_match,
            speedup_label(pt_total_ms, hp_effective_ms),
        );
    }

    section("SECTION 2  Reading The Results");
    println!("  scratch  : deep exterior filament; tiling can rescue it at moderate iteration counts, but it becomes harsher as dwell rises.");
    println!("  boundary : representative long-dwell boundary scene; if PT loses here, that marks a real practical limit.");
    println!("  period2  : best-case long-lived interior scene; 1x1 PT should show the clearest win.");
    println!();
    println!("  Use these three together:");
    println!("  - If PT loses on scratch but wins on period2, the implementation is behaving as expected.");
    println!("  - If PT also improves on boundary, tiling is doing useful real-world work.");
    println!("  - If PT loses everywhere, either the scene choices or the PT implementation need work.");
    println!();
}