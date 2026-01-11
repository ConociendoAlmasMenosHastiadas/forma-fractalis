/// iced GUI demo - Compact sidebar style with colormap controls
///
/// Run with: cargo run --example gui_demo

use iced::{
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, Space},
    Alignment, Element, Length, Sandbox, Settings,
};

pub fn main() -> iced::Result {
    ControlPanel::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(320.0, 700.0),  // Narrow sidebar width
            position: iced::window::Position::Default,
            ..Default::default()
        },
        ..Default::default()
    })
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
}

impl std::fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ColorScheme::Default => "Default",
            ColorScheme::Fire => "Fire",
            ColorScheme::Ocean => "Ocean",
            ColorScheme::Grayscale => "Grayscale",
            ColorScheme::Rainbow => "Rainbow",
        })
    }
}

// The application state
struct ControlPanel {
    // Dimension inputs
    width_input: String,
    height_input: String,
    iterations_input: String,
    
    // Colormap controls
    selected_scheme: Option<ColorScheme>,
    color_stops: Vec<String>,  // Placeholder for custom colors
    
    // View info
    center_x: f64,
    center_y: f64,
    zoom: f64,
    
    // Status message
    status_message: String,
}

impl Default for ControlPanel {
    fn default() -> Self {
        Self {
            width_input: String::from("800"),
            height_input: String::from("600"),
            iterations_input: String::from("256"),
            selected_scheme: Some(ColorScheme::Default),
            color_stops: vec![
                "RGB(0,0,0)".to_string(),
                "RGB(255,0,0)".to_string(),
                "RGB(255,255,0)".to_string(),
            ],
            center_x: -0.5,
            center_y: 0.0,
            zoom: 1.0,
            status_message: String::from("Ready"),
        }
    }
}

// All possible user interactions/events
#[derive(Debug, Clone)]
enum Message {
    WidthChanged(String),
    HeightChanged(String),
    IterationsChanged(String),
    SchemeSelected(ColorScheme),
    AddColorStop,
    SaveColormap,
    LoadColormap,
    ApplyClicked,
    ExportClicked,
    ResetClicked,
}

// Implement the Sandbox trait
impl Sandbox for ControlPanel {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Mandelbrot Controls")
    }

    // Handle all events/messages
    fn update(&mut self, message: Message) {
        match message {
            Message::WidthChanged(value) => {
                self.width_input = value;
            }
            Message::HeightChanged(value) => {
                self.height_input = value;
            }
            Message::IterationsChanged(value) => {
                self.iterations_input = value;
            }
            Message::SchemeSelected(scheme) => {
                self.selected_scheme = Some(scheme);
                self.status_message = format!("Color scheme: {:?}", scheme);
                println!("Selected color scheme: {:?}", scheme);
            }
            Message::AddColorStop => {
                // Placeholder: add a new color stop
                let new_color = format!("RGB({},{},{})", 
                    (self.color_stops.len() * 50) % 255,
                    (self.color_stops.len() * 100) % 255,
                    (self.color_stops.len() * 150) % 255
                );
                self.color_stops.push(new_color);
                self.status_message = format!("Added color stop (total: {})", self.color_stops.len());
                println!("Color stops: {:?}", self.color_stops);
            }
            Message::SaveColormap => {
                self.status_message = String::from("Save colormap (TODO)");
                println!("Save colormap feature - to be implemented");
            }
            Message::LoadColormap => {
                self.status_message = String::from("Load colormap (TODO)");
                println!("Load colormap feature - to be implemented");
            }
            Message::ApplyClicked => {
                // Parse and validate inputs
                let width: u32 = self.width_input.parse().unwrap_or(800);
                let height: u32 = self.height_input.parse().unwrap_or(600);
                let iterations: u32 = self.iterations_input.parse().unwrap_or(256);
                
                let width = width.clamp(100, 4096);
                let height = height.clamp(100, 4096);
                let iterations = iterations.clamp(10, 10000);
                
                self.status_message = format!("Applied: {}×{}", width, height);
                
                println!("\n=== Settings Applied ===");
                println!("Dimensions: {}×{}", width, height);
                println!("Iterations: {}", iterations);
                println!("View: center=({:.6}, {:.6}), zoom={:.2}x", 
                    self.center_x, self.center_y, self.zoom);
                println!("Color scheme: {:?}", self.selected_scheme);
                println!("=======================\n");
            }
            Message::ExportClicked => {
                self.status_message = String::from("Export (TODO)");
                println!("Export feature - to be implemented");
            }
            Message::ResetClicked => {
                *self = Self::default();
                self.status_message = String::from("Reset to defaults");
                println!("Reset to defaults");
            }
        }
    }

    // Build the compact sidebar UI
    fn view(&self) -> Element<Message> {
        let content = scrollable(
            column![
                // Header
                container(
                    text("Mandelbrot Controls")
                        .size(18)
                )
                .padding(10)
                .width(Length::Fill)
                .center_x(),
                
                // Dimensions section
                Self::section_header("Dimensions"),
                Self::input_row("Width", &self.width_input, Message::WidthChanged),
                Self::input_row("Height", &self.height_input, Message::HeightChanged),
                Self::input_row("Iterations", &self.iterations_input, Message::IterationsChanged),
                
                Self::divider(),
                
                // View info section
                Self::section_header("Current View"),
                Self::info_text(&format!("X: {:.6}", self.center_x)),
                Self::info_text(&format!("Y: {:.6}", self.center_y)),
                Self::info_text(&format!("Zoom: {:.2}x", self.zoom)),
                
                Self::divider(),
                
                // Colormap section
                Self::section_header("Color Scheme"),
                
                pick_list(
                    &ColorScheme::ALL[..],
                    self.selected_scheme,
                    Message::SchemeSelected,
                )
                .width(Length::Fill)
                .padding(5),
                
                Space::with_height(10),
                
                Self::section_header("Color Stops"),
                
                // Display current color stops
                column(
                    self.color_stops.iter().enumerate().map(|(i, color)| {
                        text(format!("{}. {}", i + 1, color))
                            .size(12)
                            .into()
                    }).collect::<Vec<Element<Message>>>()
                )
                .spacing(3)
                .padding(5),
                
                row![
                    button(text("Add").size(12))
                        .on_press(Message::AddColorStop)
                        .padding(5),
                    button(text("Save").size(12))
                        .on_press(Message::SaveColormap)
                        .padding(5),
                    button(text("Load").size(12))
                        .on_press(Message::LoadColormap)
                        .padding(5),
                ]
                .spacing(5)
                .width(Length::Fill),
                
                Self::divider(),
                
                // Action buttons
                button(
                    text("Apply Settings")
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                        .size(14)
                )
                .on_press(Message::ApplyClicked)
                .width(Length::Fill)
                .padding(12),
                
                button(
                    text("Reset")
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                        .size(12)
                )
                .on_press(Message::ResetClicked)
                .width(Length::Fill)
                .padding(8),
                
                button(
                    text("Export (TODO)")
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                        .size(12)
                )
                .on_press(Message::ExportClicked)
                .width(Length::Fill)
                .padding(8),
                
                Self::divider(),
                
                // Status bar
                container(
                    text(&self.status_message)
                        .size(11)
                )
                .padding(8)
                .width(Length::Fill)
                .center_x(),
            ]
            .spacing(5)
            .padding(10)
        );

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

// Helper methods for consistent UI elements
impl ControlPanel {
    fn section_header(title: &str) -> Element<'static, Message> {
        container(
            text(title)
                .size(14)
        )
        .padding([5, 0, 3, 0])
        .into()
    }
    
    fn input_row(label: &'static str, value: &str, on_change: fn(String) -> Message) -> Element<'static, Message> {
        row![
            text(label).size(12).width(70),
            text_input("", value)
                .on_input(on_change)
                .padding(4)
                .size(12)
                .width(Length::Fill),
        ]
        .spacing(5)
        .align_items(Alignment::Center)
        .into()
    }
    
    fn info_text(content: &str) -> Element<'static, Message> {
        text(content)
            .size(11)
            .into()
    }
    
    fn divider() -> Element<'static, Message> {
        Space::with_height(8).into()
    }
}
