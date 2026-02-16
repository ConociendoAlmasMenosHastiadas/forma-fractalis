//! GUI rendering trait for fractal-specific parameters
//!
//! This module provides the `FractalGUI` trait that allows each fractal type
//! to encapsulate its own parameter rendering logic, avoiding hard-coded
//! conditional sections in the main GUI code.
//!
//! # Architecture Benefits
//! - **Encapsulation**: Each fractal's GUI code lives with its implementation
//! - **Extensibility**: Adding new fractals doesn't require modifying gui.rs
//! - **Maintainability**: Parameter logic is colocated with fractal logic
//! - **Type Safety**: Trait ensures all fractals provide GUI if needed
//!
//! # Usage Pattern
//! ```ignore
//! // In gui.rs
//! fractal.render_parameters_gui(ui, params, input_state, &mut needs_redraw, ...);
//! ```

use eframe::egui;
use std::collections::HashMap;
use std::time::Instant;
use crate::app_state::InputState;

/// Trait for rendering fractal-specific parameter controls
///
/// Each fractal implements this to provide its own GUI controls (sliders,
/// text inputs, radio buttons, etc.) for its parameters.
pub trait FractalGUI {
    /// Render the parameter controls for this fractal
    ///
    /// # Arguments
    /// * `ui` - egui UI context for rendering widgets
    /// * `params` - Mutable reference to fractal parameters (HashMap<String, f64>)
    /// * `input_state` - Mutable reference to InputState (for text input fields)
    /// * `needs_redraw` - Set to true when parameters change (triggers re-render)
    /// * `debounce_timer` - Timer for debouncing text input changes
    /// * `pending_redraw` - Flag for pending debounced redraws
    ///
    /// # Implementation Notes
    /// - Use `ui.label()`, `ui.add(Slider::new())`, etc. for controls
    /// - Update `params` HashMap when sliders change
    /// - Update `input_state` fields when text inputs change
    /// - Set `*needs_redraw = true` for immediate redraws (sliders)
    /// - Call `trigger_debounced_redraw()` for text inputs
    /// - Use collapsible sections for advanced/precise controls
    ///
    /// # Default Implementation
    /// Fractals without parameters can use the default implementation which renders nothing.
    fn render_parameters_gui(
        &self,
        _ui: &mut egui::Ui,
        _params: &mut HashMap<String, f64>,
        _input_state: &mut InputState,
        _needs_redraw: &mut bool,
    ) {
        // Default: no parameters to render
    }
}

/// Helper function to trigger a debounced redraw (for text inputs)
///
/// This prevents excessive re-renders while the user is still typing.
/// Call this when text input changes, rather than setting needs_redraw directly.
/// Access input_state fields directly: input_state.debounce_timer, input_state.pending_redraw
pub fn trigger_debounced_redraw(timer: &mut Option<Instant>, pending: &mut bool) {
    *timer = Some(Instant::now());
    *pending = true;
}
