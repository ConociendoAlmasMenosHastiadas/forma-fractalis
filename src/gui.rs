/// Integrated Mandelbrot GUI with iced
/// 
/// This module contains the control panel GUI that integrates with the fractal renderer

use iced::{
    executor,
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, Space},
    Alignment, Application, Command, Element, Length, Settings, Theme,
};
use std::sync::{Arc, Mutex};
use crate::fractal::MandelbrotView;

static VIEW: once_cell::sync::OnceCell<Arc<Mutex<MandelbrotView>>> = once_cell::sync::OnceCell::new();

/// Launch the GUI application with shared view
pub fn run_with_view(view: Arc<Mutex<MandelbrotView>>) -> iced::Result {
    VIEW.set(view).ok();
    MandelbrotApp::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(320.0, 750.0),
            position: iced::window::Position::Specific(iced::Point::new(50.0, 50.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}

// Color schemes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
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

// Application state
struct MandelbrotApp {
    // UI inputs
    width_input: String,
    height_input: String,
    iterations_input: String,
    
    // Colormap
    selected_scheme: Option<ColorScheme>,
    color_stops: Vec<String>,
    
    // Status
    status_message: String,
}

// Messages
#[derive(Debug, Clone)]
enum Message {
    WidthChanged(String),
    HeightChanged(String),
    IterationsChanged(String),
    SchemeSelected(ColorScheme),
    AddColorStop,
    SaveColormap,
    LoadColormap,
    ApplySettings,
    ExportImage,
    ResetView,
}

impl Application for MandelbrotApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
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
                status_message: String::from("Ready - Fractal renderer integrated"),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Mandelbrot Controls")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
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
            }
            Message::AddColorStop => {
                let new_color = format!("RGB({},{},{})", 
                    (self.color_stops.len() * 50) % 255,
                    (self.color_stops.len() * 100) % 255,
                    (self.color_stops.len() * 150) % 255
                );
                self.color_stops.push(new_color);
                self.status_message = format!("Added color stop ({} total)", self.color_stops.len());
            }
            Message::SaveColormap => {
                self.status_message = String::from("Save colormap (TODO)");
            }
            Message::LoadColormap => {
                self.status_message = String::from("Load colormap (TODO)");
            }
            Message::ApplySettings => {
                let width: u32 = self.width_input.parse().unwrap_or(800).clamp(100, 4096);
                let height: u32 = self.height_input.parse().unwrap_or(600).clamp(100, 4096);
                let iterations: u32 = self.iterations_input.parse().unwrap_or(256).clamp(10, 10000);
                
                // Update the view (note: dimensions require restart)
                if let Some(view_arc) = VIEW.get() {
                    if let Ok(view) = view_arc.lock() {
                        if view.width != width || view.height != height {
                            self.status_message = format!("Dimensions require restart: {}×{}", width, height);
                        } else {
                            self.status_message = format!("Applied: {} iterations", iterations);
                        }
                        
                        println!("\n=== Settings Applied ===");
                        println!("Requested: {}×{}, {} iter", width, height, iterations);
                        println!("Current: {}×{}", view.width, view.height);
                        println!("Center: ({:.6}, {:.6})", view.center_x, view.center_y);
                        println!("Zoom: {:.2}x", view.zoom);
                        println!("Color scheme: {:?}", self.selected_scheme);
                        println!("=======================\n");
                    }
                } else {
                    self.status_message = String::from("View not initialized");
                }
            }
            Message::ExportImage => {
                self.status_message = String::from("Export (TODO)");
            }
            Message::ResetView => {
                if let Some(view_arc) = VIEW.get() {
                    if let Ok(mut view) = view_arc.lock() {
                        view.reset();
                    }
                }
                self.width_input = String::from("800");
                self.height_input = String::from("600");
                self.iterations_input = String::from("256");
                self.selected_scheme = Some(ColorScheme::Default);
                self.status_message = String::from("Reset to defaults");
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let view_info = if let Some(view_arc) = VIEW.get() {
            if let Ok(view) = view_arc.lock() {
                (view.center_x, view.center_y, view.zoom, view.width, view.height)
            } else {
                (-0.5, 0.0, 1.0, 800, 600)
            }
        } else {
            (-0.5, 0.0, 1.0, 800, 600)
        };

        let content = scrollable(
            column![
                // Header
                container(text("Mandelbrot Controls").size(18))
                    .padding(10)
                    .width(Length::Fill)
                    .center_x(),
                
                // Dimensions
                Self::section_header("Dimensions"),
                Self::input_row("Width", &self.width_input, Message::WidthChanged),
                Self::input_row("Height", &self.height_input, Message::HeightChanged),
                Self::input_row("Iterations", &self.iterations_input, Message::IterationsChanged),
                
                Self::divider(),
                
                // View info
                Self::section_header("Current View"),
                Self::info_text(&format!("X: {:.6}", view_info.0)),
                Self::info_text(&format!("Y: {:.6}", view_info.1)),
                Self::info_text(&format!("Zoom: {:.2}x", view_info.2)),
                Self::info_text(&format!("Size: {}×{}", view_info.3, view_info.4)),
                
                Self::divider(),
                
                // Colormap
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
                
                // Actions
                button(text("Apply Settings").horizontal_alignment(iced::alignment::Horizontal::Center).size(14))
                    .on_press(Message::ApplySettings)
                    .width(Length::Fill)
                    .padding(12),
                
                button(text("Reset").horizontal_alignment(iced::alignment::Horizontal::Center).size(12))
                    .on_press(Message::ResetView)
                    .width(Length::Fill)
                    .padding(8),
                
                button(text("Export (TODO)").horizontal_alignment(iced::alignment::Horizontal::Center).size(12))
                    .on_press(Message::ExportImage)
                    .width(Length::Fill)
                    .padding(8),
                
                Self::divider(),
                
                // Status
                container(text(&self.status_message).size(11))
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

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

// Helper methods
impl MandelbrotApp {
    fn section_header(title: &str) -> Element<'static, Message> {
        container(text(title).size(14))
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
        text(content).size(11).into()
    }
    
    fn divider() -> Element<'static, Message> {
        Space::with_height(8).into()
    }
}
