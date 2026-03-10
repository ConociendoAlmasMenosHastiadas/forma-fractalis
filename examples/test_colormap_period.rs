//! Display Egyptian Echo colormap details

use scala_chromatica::io as colorschemes_io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let colormap = colorschemes_io::load_colormap("Egyptian Echo")?;
    
    println!("=== Egyptian Echo ColorMap ===\n");
    println!("Number of stops: {}", colormap.stops.len());
    
    for (i, stop) in colormap.stops.iter().enumerate() {
        println!("Stop {}: position={:.3}, color=({}, {}, {})", 
            i, 
            stop.position, 
            stop.color.r, 
            stop.color.g, 
            stop.color.b
        );
    }
    
    println!("\n=== Sample Colors with Different Periods ===");
    
    // Sample at various points
    for period in [1.0, 2.0, 4.0].iter() {
        println!("\nPeriod = {:.1}:", period);
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let adjusted_t = (t * period) % 1.0;
            let color = colormap.get_color(adjusted_t);
            println!("  t={:.1} (adj={:.3}) -> RGB({}, {}, {})", 
                t, adjusted_t, color.r, color.g, color.b);
        }
    }
    
    Ok(())
}
