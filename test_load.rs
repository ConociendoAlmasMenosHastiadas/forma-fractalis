use forma_fractalis::colorschemes_io;
fn main() {
    match colorschemes_io::load_builtin_colormap("Electric Neon") {
        Ok(cm) => println!("Loaded: {} with {} stops", cm.name, cm.stops.len()),
        Err(e) => println!("Error: {}", e),
    }
}
