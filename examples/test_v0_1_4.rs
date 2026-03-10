//! Verify scala-chromatica v0.1.4 colorstop period fix.
//!
//! With the old formula `(iter % period) / period`, t never reached 1.0 so
//! endpoint colors (e.g. gold in Egyptian Echo) were never sampled.
//!
//! The new inclusive formula `(iter % period) / (period - 1)` hits both endpoints.
//!
//! Run: cargo run --example test_v0_1_4

use scala_chromatica::color_from_iterations;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let egyptian = scala_chromatica::io::load_builtin_colormap("Egyptian Echo")?;

    println!("=== Egyptian Echo period=2 fix verification ===");
    println!("Blue endpoint: RGB(53, 80, 112)  Gold endpoint: RGB(234, 172, 0)");
    println!();

    let mut saw_blue = false;
    let mut saw_gold = false;

    for iter in 0u32..6 {
        let color = color_from_iterations(
            iter,
            100,
            &egyptian,
            true,
            2,
            false,
            [0, 0, 0],
            false,
        );
        let label = match (iter % 2, color.r > 200) {
            (0, _) => "blue endpoint expected",
            (1, true) => "gold endpoint expected",
            _ => "UNEXPECTED COLOR",
        };
        println!("iter={} -> RGB({}, {}, {})  [{}]", iter, color.r, color.g, color.b, label);

        if iter % 2 == 0 && color.r < 100 {
            saw_blue = true;
        }
        if iter % 2 == 1 && color.r > 200 {
            saw_gold = true;
        }
    }

    println!();
    if saw_blue && saw_gold {
        println!("PASS: Both endpoints sampled correctly.");
    } else {
        println!("FAIL: Missing endpoint(s). saw_blue={}, saw_gold={}", saw_blue, saw_gold);
        std::process::exit(1);
    }

    println!();
    println!("=== period=5 sampling (should show 5 distinct t values) ===");
    for iter in 0u32..5 {
        let color = color_from_iterations(iter, 100, &egyptian, true, 5, false, [0, 0, 0], false);
        println!("iter={} -> RGB({}, {}, {})", iter, color.r, color.g, color.b);
    }

    Ok(())
}
