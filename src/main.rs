use eframe::egui;
use mandelrust::{
    fractal::MandelbrotView, 
    rendering::render_mandelbrot,
    colorschemes::{ColorScheme, ColorMap},
};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1580.0, 750.0])  // 300px sidebar + 1280x720 (16:9) display area
            .with_title("Mandelbrot Set Explorer"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Mandelbrot Set Explorer",
        options,
        Box::new(|cc| {
            // Enable high DPI scaling
            cc.egui_ctx.set_pixels_per_point(1.0);
            Box::<MandelbrotApp>::default()
        }),
    )
}

struct MandelbrotApp {
    // View state
    view: MandelbrotView,
    
    // UI inputs
    width_input: String,
    height_input: String,
    iterations_input: String,
    
    // Colormap
    selected_scheme: ColorScheme,
    colormap: ColorMap,
    
    // Fractal texture
    fractal_texture: Option<egui::TextureHandle>,
    needs_redraw: bool,
    
    // Mouse interaction
    is_dragging: bool,
    zoom_square_center: Option<egui::Pos2>,
    zoom_square_size: f32,
    
    // Status
    status_message: String,
}

impl Default for MandelbrotApp {
    fn default() -> Self {
        Self {
            view: MandelbrotView::new(1280, 720),
            width_input: String::from("1280"),
            height_input: String::from("720"),
            iterations_input: String::from("256"),
            selected_scheme: ColorScheme::Default,
            color_stops: vec![
                "RGB(0,0,0)".to_string(),
                "RGB(255,0,0)".to_string(),
                "RGB(255,255,0)".to_string(),
            ],
            fractal_texture: None,
            needs_redraw: true,
            is_dragging: false,
            zoom_square_center: None,
            zoom_square_size: 200.0,
            status_message: String::from("Ready - Click+drag to position zoom, scroll to resize"),
        }
    }
}

impl MandelbrotApp {
    fn render_fractal(&mut self, ctx: &egui::Context) {
        // Render to buffer
        let width = self.view.width as usize;
        let height = self.view.height as usize;
        let mut buffer = vec![0u8; width * height * 4];
        render_mandelbrot(&mut buffer, &self.view, &self.colormap);
        
        // Convert to egui ColorImage
        let selected_scheme = ColorScheme::Default;
        Self {
            view: MandelbrotView::new(1280, 720),
            width_input: String::from("1280"),
            height_input: String::from("720"),
            iterations_input: String::from("256"),
            selected_scheme,
            colormap: selected_scheme.to_colormap()date or create texture
        if let Some(texture) = &mut self.fractal_texture {
            texture.set(color_image, egui::TextureOptions::NEAREST);
        } else {
            self.fractal_texture = Some(ctx.load_texture(
                "mandelbrot",
                color_image,
                egui::TextureOptions::NEAREST,
            ));
        }
        
        self.needs_redraw = false;
    }
}

impl eframe::App for MandelbrotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Render fractal if needed
        if self.needs_redraw {
            self.render_fractal(ctx);
        }
        
        // Left sidebar with controls
        egui::SidePanel::left("controls")
            .default_width(300.0)
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Controls");
                        ui.add_space(10.0);
                        
                        // Preview Window Dimensions
                        ui.label(egui::RichText::new("Preview Window Dimensions").strong());
                        ui.add_space(5.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("Width:");
                            ui.add_space(28.0);
                            ui.add(egui::TextEdit::singleline(&mut self.width_input).desired_width(120.0));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Height:");
                            ui.add_space(24.0);
                            ui.add(egui::TextEdit::singleline(&mut self.height_input).desired_width(120.0));
                        });
                        
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("These controls will not resize the preview window but they will control the aspect ratio and performance of the preview. See rendering below for high-res output.").small().italics().color(egui::Color32::GRAY));
                        
                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        // Fractal Settings
                        ui.label(egui::RichText::new("Fractal Settings").strong());
                        ui.add_space(5.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("Iterations:");
                            ui.add_space(5.0);
                            ui.add(egui::TextEdit::singleline(&mut self.iterations_input).desired_width(120.0));
                        });
                        
                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        // Current View
                        ui.label(egui::RichText::new("Current View").strong());
                        ui.add_space(5.0);
                        
                        ui.label(format!("X: {:.6}", self.view.center_x));
                        ui.label(format!("Y: {:.6}", self.view.center_y));
                        ui.label(format!("Zoom: {:.2}x", self.view.zoom));
                        ui.label(format!("Size: {}×{}", self.view.width, self.view.height));
                        
                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        // Color Scheme
                        ui.label(egui::RichText::new("Color Scheme").strong());
                        ui.add_space(5.0);
                        
                        egui::ComboBox::from_label("")
                            .selected_text(self.selected_scheme.as_str())
                            .show_ui(ui, |ui| {
                                for scheme in ColorScheme::ALL.iter() {
                                    if ui.selectable_value(&mut self.selected_scheme, *scheme, scheme.as_str()).clicked() {
                                        self.colormap = scheme.to_colormap();
                                        self.needs_redraw = true;
                                    }
                                }
                            });
                        
                        ui.add_space(15.0);
                        
                        // Color Stops
                        ui.label(egui::RichText::new("Color Stops").strong());
                        ui.add_space(5.0);
                        
                        for (i, stop) in self.colormap.stops().iter().enumerate() {
                            ui.label(format!("{}. {} at {:.2}", i + 1, stop.color, stop.position));
                        }
                        
                        ui.add_space(5.0);
                        
                        ui.horizontal(|ui| {
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
                        
                        // Actions
                        if ui.add_sized([ui.available_width(), 40.0], egui::Button::new("Apply Settings")).clicked() {
                            let width: u32 = self.width_input.parse().unwrap_or(800).clamp(100, 4096);
                            let height: u32 = self.height_input.parse().unwrap_or(600).clamp(100, 4096);
                            let iterations: u32 = self.iterations_input.parse().unwrap_or(256).clamp(10, 10000);
                            
                            self.view.width = width;
                            self.view.height = height;
                            self.needs_redraw = true;
                            self.status_message = format!("Applied: {}×{}, {} iter", width, height, iterations);
                        }
                        
                        ui.add_space(5.0);
                        
                        if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Reset View")).clicked() {
                            self.view.reset();
                            self.needs_redraw = true;
                            self.status_message = String::from("Reset to defaults");
                        }
                        
                        ui.add_space(5.0);
                        
                        if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Export (TODO)")).clicked() {
                            self.status_message = String::from("Export (TODO)");
                        }
                        
                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        // Status
                        ui.label(egui::RichText::new(&self.status_message).small().italics());
                    });
                });
            });
        
        // Main fractal display
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.fractal_texture {
                let available_size = ui.available_size();
                let texture_size = texture.size_vec2();
                let scale = (available_size.x / texture_size.x).min(available_size.y / texture_size.y);
                let display_size = texture_size * scale;
                
                let (rect, response) = ui.allocate_exact_size(display_size, egui::Sense::click_and_drag());
                
                // Handle scroll wheel for zoom square resize
                let scroll_delta = ui.input(|i| i.scroll_delta.y);
                if scroll_delta != 0.0 && self.is_dragging {
                    self.zoom_square_size = (self.zoom_square_size + scroll_delta * 2.0).max(20.0).min(2000.0);
                    self.status_message = format!("Zoom size: {:.0}px", self.zoom_square_size);
                }
                
                // Handle mouse interactions
                if response.drag_started() {
                    self.is_dragging = true;
                    self.zoom_square_center = response.interact_pointer_pos();
                }
                
                // Update square position while dragging
                if self.is_dragging {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.zoom_square_center = Some(pos);
                    }
                }
                
                if response.drag_released() && self.is_dragging {
                    self.is_dragging = false;
                    if let Some(center) = self.zoom_square_center {
                        // Calculate zoom region
                        let center_rel = (center - rect.min) / scale;
                        let square_size_rel = self.zoom_square_size / scale;
                        
                        let center_x = center_rel.x as u32;
                        let center_y = center_rel.y as u32;
                        
                        // Calculate zoom factor based on square size relative to image size
                        let zoom_factor = texture_size.x / square_size_rel;
                        
                        self.view.zoom_at(center_x, center_y, zoom_factor as f64);
                        self.needs_redraw = true;
                        self.status_message = format!("Zoomed to {:.2}x", self.view.zoom);
                    }
                    self.zoom_square_center = None;
                }
                
                // Right click to zoom out
                if response.secondary_clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let rel_pos = (pos - rect.min) / scale;
                        let pixel_x = rel_pos.x as u32;
                        let pixel_y = rel_pos.y as u32;
                        
                        self.view.zoom_at(pixel_x, pixel_y, 0.5);
                        self.needs_redraw = true;
                        self.status_message = format!("Zoomed out to {:.2}x", self.view.zoom);
                    }
                }
                
                // Draw the fractal
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
                
                // Draw zoom rectangle if dragging (matching aspect ratio)
                if self.is_dragging {
                    if let Some(center) = self.zoom_square_center {
                        // Calculate rectangle size maintaining aspect ratio
                        let aspect_ratio = self.view.width as f32 / self.view.height as f32;
                        let rect_width = self.zoom_square_size;
                        let rect_height = self.zoom_square_size / aspect_ratio;
                        
                        let zoom_rect = egui::Rect::from_center_size(
                            center,
                            egui::vec2(rect_width, rect_height)
                        );
                        ui.painter().rect_stroke(
                            zoom_rect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 255)), // Magenta
                        );
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
            }
        });
        
        // Request repaint for smooth interaction
        if self.is_dragging {
            ctx.request_repaint();
        }
    }
}
