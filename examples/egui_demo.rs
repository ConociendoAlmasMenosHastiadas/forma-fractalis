/// egui demo - Compact sidebar style with colormap controls
///
/// Run with: cargo run --example egui_demo

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 700.0])
            .with_position([50.0, 50.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Mandelbrot Controls",
        options,
        Box::new(|_cc| Box::<ControlPanel>::default()),
    )
}

// Available color schemes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorScheme {
    Default,
    Fire,
    Ocean,
    Grayscale,
    Rainbow,
}

impl ColorScheme {
    const ALL: [ColorScheme; 5] = [
        ColorScheme::Default,
        ColorScheme::Fire,
        ColorScheme::Ocean,
        ColorScheme::Grayscale,
        ColorScheme::Rainbow,
    ];
    
    fn as_str(&self) -> &'static str {
        match self {
            ColorScheme::Default => "Default",
            ColorScheme::Fire => "Fire",
            ColorScheme::Ocean => "Ocean",
            ColorScheme::Grayscale => "Grayscale",
            ColorScheme::Rainbow => "Rainbow",
        }
    }
}

struct ControlPanel {
    // Dimension inputs
    width_input: String,
    height_input: String,
    iterations_input: String,
    
    // View state
    center_x: f64,
    center_y: f64,
    zoom: f64,
    
    // Colormap
    selected_scheme: ColorScheme,
    color_stops: Vec<String>,
    
    // Status
    status_message: String,
}

impl Default for ControlPanel {
    fn default() -> Self {
        Self {
            width_input: String::from("800"),
            height_input: String::from("600"),
            iterations_input: String::from("256"),
            center_x: -0.5,
            center_y: 0.0,
            zoom: 1.0,
            selected_scheme: ColorScheme::Default,
            color_stops: vec![
                "RGB(0,0,0)".to_string(),
                "RGB(255,0,0)".to_string(),
                "RGB(255,255,0)".to_string(),
            ],
            status_message: String::from("Ready"),
        }
    }
}

impl eframe::App for ControlPanel {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    // Header
                    ui.heading("Mandelbrot Controls");
                    ui.add_space(10.0);
                    
                    // Dimensions section
                    ui.label(egui::RichText::new("Dimensions").strong());
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.add_space(28.0);
                        ui.add(egui::TextEdit::singleline(&mut self.width_input).desired_width(150.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        ui.add_space(24.0);
                        ui.add(egui::TextEdit::singleline(&mut self.height_input).desired_width(150.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Iterations:");
                        ui.add_space(5.0);
                        ui.add(egui::TextEdit::singleline(&mut self.iterations_input).desired_width(150.0));
                    });
                    
                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(10.0);
                    
                    // Current View section
                    ui.label(egui::RichText::new("Current View").strong());
                    ui.add_space(5.0);
                    
                    ui.label(format!("X: {:.6}", self.center_x));
                    ui.label(format!("Y: {:.6}", self.center_y));
                    ui.label(format!("Zoom: {:.2}x", self.zoom));
                    ui.label(format!("Size: {}×{}", 
                        self.width_input.parse::<u32>().unwrap_or(800),
                        self.height_input.parse::<u32>().unwrap_or(600)));
                    
                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(10.0);
                    
                    // Color Scheme section
                    ui.label(egui::RichText::new("Color Scheme").strong());
                    ui.add_space(5.0);
                    
                    egui::ComboBox::from_label("")
                        .selected_text(self.selected_scheme.as_str())
                        .show_ui(ui, |ui| {
                            for scheme in ColorScheme::ALL.iter() {
                                ui.selectable_value(&mut self.selected_scheme, *scheme, scheme.as_str());
                            }
                        });
                    
                    ui.add_space(15.0);
                    
                    // Color Stops section
                    ui.label(egui::RichText::new("Color Stops").strong());
                    ui.add_space(5.0);
                    
                    for (i, color) in self.color_stops.iter().enumerate() {
                        ui.label(format!("{}. {}", i + 1, color));
                    }
                    
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Add").clicked() {
                            let new_color = format!("RGB({},{},{})", 
                                (self.color_stops.len() * 50) % 255,
                                (self.color_stops.len() * 100) % 255,
                                (self.color_stops.len() * 150) % 255
                            );
                            self.color_stops.push(new_color);
                            self.status_message = format!("Added color stop ({} total)", self.color_stops.len());
                        }
                        
                        if ui.button("Save").clicked() {
                            self.status_message = String::from("Save colormap (TODO)");
                        }
                        
                        if ui.button("Load").clicked() {
                            self.status_message = String::from("Load colormap (TODO)");
                        }
                    });
                    
                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(10.0);
                    
                    // Action buttons
                    if ui.add_sized([ui.available_width(), 40.0], egui::Button::new("Apply Settings")).clicked() {
                        let width: u32 = self.width_input.parse().unwrap_or(800).clamp(100, 4096);
                        let height: u32 = self.height_input.parse().unwrap_or(600).clamp(100, 4096);
                        let iterations: u32 = self.iterations_input.parse().unwrap_or(256).clamp(10, 10000);
                        
                        self.status_message = format!("Applied: {}×{}, {} iter", width, height, iterations);
                        
                        println!("\n=== Settings Applied ===");
                        println!("Dimensions: {}×{}", width, height);
                        println!("Iterations: {}", iterations);
                        println!("Center: ({:.6}, {:.6})", self.center_x, self.center_y);
                        println!("Zoom: {:.2}x", self.zoom);
                        println!("Color scheme: {:?}", self.selected_scheme);
                        println!("=======================\n");
                    }
                    
                    ui.add_space(5.0);
                    
                    if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Reset")).clicked() {
                        self.width_input = String::from("800");
                        self.height_input = String::from("600");
                        self.iterations_input = String::from("256");
                        self.center_x = -0.5;
                        self.center_y = 0.0;
                        self.zoom = 1.0;
                        self.selected_scheme = ColorScheme::Default;
                        self.status_message = String::from("Reset to defaults");
                    }
                    
                    ui.add_space(5.0);
                    
                    if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Export (TODO)")).clicked() {
                        self.status_message = String::from("Export (TODO)");
                    }
                    
                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(10.0);
                    
                    // Status message
                    ui.label(egui::RichText::new(&self.status_message).small().italics());
                });
            });
        });
    }
}
