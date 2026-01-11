// Example demonstrating ColorMap save/load functionality

use mandelrust::colorschemes::{Color, ColorMap, ColorStop};
use mandelrust::colorschemes_io::{
    delete_custom_colormap, export_builtin_colormap, get_colormaps_directory,
    list_available_colormaps, load_builtin_colormap, load_colormap, save_colormap,
};

fn main() {
    println!("=== ColorMap Save/Load Example ===\n");

    // Show where custom colormaps are stored
    match get_colormaps_directory() {
        Ok(dir) => println!("Custom colormaps directory: {}\n", dir.display()),
        Err(e) => println!("Error getting directory: {}\n", e),
    }

    // Load a built-in colormap
    println!("1. Loading built-in 'Fire' colormap...");
    match load_builtin_colormap("Fire") {
        Ok(colormap) => {
            println!(
                "   ✓ Loaded '{}' with {} stops",
                colormap.name,
                colormap.stops.len()
            );
            for (i, stop) in colormap.stops.iter().enumerate() {
                println!(
                    "     Stop {}: position={:.2}, color={}",
                    i, stop.position, stop.color
                );
            }
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // Create a custom colormap
    println!("2. Creating and saving a custom 'Purple Dream' colormap...");
    let custom_colormap = ColorMap::new(
        "Purple Dream".to_string(),
        vec![
            ColorStop::new(0.0, Color::black()),
            ColorStop::new(0.33, Color::new(75, 0, 130)), // Indigo
            ColorStop::new(0.67, Color::new(138, 43, 226)), // Blue Violet
            ColorStop::new(1.0, Color::new(255, 192, 203)), // Pink
        ],
    );

    match save_colormap(&custom_colormap) {
        Ok(path) => println!("   ✓ Saved to: {}", path.display()),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // List all available colormaps
    println!("3. Listing all available colormaps...");
    match list_available_colormaps() {
        Ok(colormaps) => {
            for info in colormaps {
                let location = if info.is_builtin {
                    "(built-in)"
                } else {
                    "(custom)"
                };
                println!("   - {} {}", info.name, location);
            }
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // Load using the generic load_colormap function
    println!("4. Loading 'Purple Dream' using load_colormap()...");
    match load_colormap("Purple Dream") {
        Ok(colormap) => {
            println!(
                "   ✓ Loaded '{}' with {} stops",
                colormap.name,
                colormap.stops.len()
            );
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // Export a built-in colormap to customize it
    println!("5. Exporting 'Ocean' built-in to custom directory...");
    match export_builtin_colormap("Ocean") {
        Ok(path) => println!("   ✓ Exported to: {}", path.display()),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // Clean up: delete the custom colormap we created
    println!("6. Cleaning up: deleting 'Purple Dream'...");
    match delete_custom_colormap("Purple Dream") {
        Ok(_) => println!("   ✓ Deleted successfully"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    println!("\n=== Example Complete ===");
}
