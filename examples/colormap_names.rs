// Example showing how to use color names in ColorStops

use mandelrust::colorschemes_io::load_builtin_colormap;

fn main() {
    println!("=== ColorStop Name Feature Demo ===\n");

    // Load colormaps with named color stops
    let colormaps = vec![
        "Academic",
        "Mint Lavender",
        "Coral Sunset",
        "Olive Symmetry",
        "Orchid Garden",
    ];

    for name in colormaps {
        println!("{}:", name);
        match load_builtin_colormap(name) {
            Ok(colormap) => {
                for (i, stop) in colormap.stops.iter().enumerate() {
                    let color_name = stop
                        .name
                        .as_ref()
                        .map(|n| format!(" ({})", n))
                        .unwrap_or_default();

                    println!(
                        "  Stop {}: pos={:.2} {} {}",
                        i, stop.position, stop.color, color_name
                    );
                }
            }
            Err(e) => println!("  Error: {}", e),
        }
        println!();
    }

    println!("The 'name' field in ColorStop is optional and can be:");
    println!("  • Displayed in UI tooltips");
    println!("  • Shown in color picker legends");
    println!("  • Used for documentation");
    println!("  • Omitted (None) for unnamed colors");
}
