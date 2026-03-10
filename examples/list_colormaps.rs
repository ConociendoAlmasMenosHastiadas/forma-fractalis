//! List all available colormaps (built-in and custom)

use scala_chromatica::io as colorschemes_io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Available ColorMaps ===\n");
    
    let colormaps = colorschemes_io::list_available_colormaps()?;
    
    if colormaps.is_empty() {
        println!("No colormaps found!");
        return Ok(());
    }
    
    for (i, info) in colormaps.iter().enumerate() {
        println!("{}. {} ({})", 
            i + 1, 
            info.name, 
            if info.is_builtin { "built-in" } else { "custom" }
        );
    }
    
    println!("\n=== ColorMap Directory ===");
    match colorschemes_io::get_colormaps_directory() {
        Ok(path) => println!("Custom colormaps directory: {}", path.display()),
        Err(e) => println!("Error getting directory: {}", e),
    }
    
    Ok(())
}
