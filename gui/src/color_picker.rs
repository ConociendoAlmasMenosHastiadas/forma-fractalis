//! HSV Color Picker Widget
//!
//! An interactive HSV color picker inspired by chromator, designed to embed
//! inline in egui panels. Provides a 2D saturation-value plane, a hue bar,
//! hex input, and a live color preview swatch.
//!
//! Replaces the old per-channel RGB sliders with a proper visual picker.

use eframe::egui;

// ── Color conversions ──────────────────────────────────────────────

/// HSV to RGB (all values 0.0-1.0)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = (h * 360.0) / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    (r1 + m, g1 + m, b1 + m)
}

/// RGB (0.0-1.0) to HSV (h 0.0-1.0, s 0.0-1.0, v 0.0-1.0)
fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        (((g - b) / delta) % 6.0 + 6.0) % 6.0 / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };
    let s = if max == 0.0 { 0.0 } else { delta / max };
    (h, s, max)
}

// ── Widget state ───────────────────────────────────────────────────

/// Persistent state for one color picker instance.
///
/// Each color picker that appears in the UI needs its own `ColorPickerState`
/// to hold cached textures and HSV working values.
pub struct ColorPickerState {
    hue: f32,
    sat: f32,
    val: f32,
    sv_texture: Option<egui::TextureHandle>,
    sv_texture_hue: f32,
    hue_texture: Option<egui::TextureHandle>,
    hex_input: String,
    rgb_input: [String; 3],
}

impl Default for ColorPickerState {
    fn default() -> Self {
        Self {
            hue: 0.0,
            sat: 0.0,
            val: 0.0,
            sv_texture: None,
            sv_texture_hue: -1.0,
            hue_texture: None,
            hex_input: String::from("000000"),
            rgb_input: [String::from("0"), String::from("0"), String::from("0")],
        }
    }
}

impl ColorPickerState {
    /// Sync the picker's HSV state from an external RGB value.
    /// Call this once when opening the picker or when the external value changes.
    pub fn set_rgb(&mut self, rgb: [u8; 3]) {
        let (h, s, v) = rgb_to_hsv(
            rgb[0] as f32 / 255.0,
            rgb[1] as f32 / 255.0,
            rgb[2] as f32 / 255.0,
        );
        self.hue = h;
        self.sat = s;
        self.val = v;
        self.hex_input = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        self.rgb_input = [
            rgb[0].to_string(),
            rgb[1].to_string(),
            rgb[2].to_string(),
        ];
    }

    /// Sync the text inputs from the current HSV state (after SV/hue interaction).
    fn sync_text_from_hsv(&mut self) {
        let [r, g, b] = self.get_rgb();
        self.hex_input = format!("{:02X}{:02X}{:02X}", r, g, b);
        self.rgb_input = [r.to_string(), g.to_string(), b.to_string()];
    }

    /// Get the current color as RGB bytes.
    pub fn get_rgb(&self) -> [u8; 3] {
        let (r, g, b) = hsv_to_rgb(self.hue, self.sat, self.val);
        [
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
        ]
    }
}

// ── Main widget entry point ────────────────────────────────────────

/// Render an inline HSV color picker.
///
/// Returns `true` if the color was changed this frame.
/// The current color is read from / written to `rgb`.
///
/// `id_salt` must be unique per picker instance to avoid texture/state collisions.
pub fn show_color_picker(
    ui: &mut egui::Ui,
    state: &mut ColorPickerState,
    rgb: &mut [u8; 3],
    id_salt: &str,
) -> bool {
    let mut changed = false;

    // Sync state -> HSV on first frame (detect external changes)
    let current_rgb = state.get_rgb();
    if current_rgb != *rgb {
        state.set_rgb(*rgb);
    }

    let panel_width = ui.available_width().min(300.0);

    // ── Color swatch (current color) ───────────────────────────
    let swatch_height = 24.0;
    let (swatch_rect, _) =
        ui.allocate_exact_size(egui::vec2(panel_width, swatch_height), egui::Sense::hover());
    let (r, g, b) = hsv_to_rgb(state.hue, state.sat, state.val);
    let preview_color = egui::Color32::from_rgb(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    );
    ui.painter().rect_filled(swatch_rect, 4.0, preview_color);
    ui.painter()
        .rect_stroke(
            swatch_rect,
            4.0,
            egui::Stroke::new(1.0, egui::Color32::GRAY),
            egui::StrokeKind::Middle,
        );

    ui.add_space(4.0);

    // ── SV plane ───────────────────────────────────────────────
    let sv_height = panel_width * 0.75;
    changed |= draw_sv_plane(ui, state, panel_width, sv_height, id_salt);

    ui.add_space(4.0);

    // ── Hue bar ────────────────────────────────────────────────
    changed |= draw_hue_bar(ui, state, panel_width, id_salt);

    ui.add_space(6.0);

    // ── Hex input + RGB readout ────────────────────────────────
    changed |= draw_hex_and_rgb(ui, state, id_salt);

    // Write back
    if changed {
        *rgb = state.get_rgb();
    }

    changed
}

// ── SV plane ───────────────────────────────────────────────────────

fn draw_sv_plane(
    ui: &mut egui::Ui,
    state: &mut ColorPickerState,
    width: f32,
    height: f32,
    id_salt: &str,
) -> bool {
    let mut changed = false;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click_and_drag());

    if response.clicked() || response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() {
            state.sat = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            state.val = 1.0 - ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
            state.sync_text_from_hsv();
            changed = true;
        }
    }

    // Regenerate texture when hue changes
    let tex_size: usize = 128;
    if state.sv_texture.is_none() || state.sv_texture_hue != state.hue {
        let mut pixels = Vec::with_capacity(tex_size * tex_size);
        for row in 0..tex_size {
            for col in 0..tex_size {
                let s = col as f32 / (tex_size - 1) as f32;
                let v = 1.0 - row as f32 / (tex_size - 1) as f32;
                let (cr, cg, cb) = hsv_to_rgb(state.hue, s, v);
                pixels.push(egui::Color32::from_rgb(
                    (cr * 255.0) as u8,
                    (cg * 255.0) as u8,
                    (cb * 255.0) as u8,
                ));
            }
        }
        let image = egui::ColorImage::new([tex_size, tex_size], pixels);
        let tex_name = format!("sv_plane_{}", id_salt);
        let tex = ui
            .ctx()
            .load_texture(tex_name, image, egui::TextureOptions::LINEAR);
        state.sv_texture = Some(tex);
        state.sv_texture_hue = state.hue;
    }

    // Paint the texture
    if let Some(tex) = &state.sv_texture {
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter()
            .image(tex.id(), rect, uv, egui::Color32::WHITE);
    }

    // Crosshair
    let cx = rect.left() + state.sat * rect.width();
    let cy = rect.top() + (1.0 - state.val) * rect.height();
    let crosshair_color = if state.val > 0.5 {
        egui::Color32::BLACK
    } else {
        egui::Color32::WHITE
    };
    ui.painter().circle_stroke(
        egui::pos2(cx, cy),
        6.0,
        egui::Stroke::new(2.0, crosshair_color),
    );

    changed
}

// ── Hue bar ────────────────────────────────────────────────────────

fn draw_hue_bar(
    ui: &mut egui::Ui,
    state: &mut ColorPickerState,
    width: f32,
    id_salt: &str,
) -> bool {
    let mut changed = false;
    let bar_height = 20.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, bar_height), egui::Sense::click_and_drag());

    if response.clicked() || response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() {
            state.hue = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            state.sync_text_from_hsv();
            changed = true;
        }
    }

    // Generate hue texture once
    if state.hue_texture.is_none() {
        let tex_width: usize = 256;
        let mut pixels = Vec::with_capacity(tex_width);
        for i in 0..tex_width {
            let h = i as f32 / (tex_width - 1) as f32;
            let (cr, cg, cb) = hsv_to_rgb(h, 1.0, 1.0);
            pixels.push(egui::Color32::from_rgb(
                (cr * 255.0) as u8,
                (cg * 255.0) as u8,
                (cb * 255.0) as u8,
            ));
        }
        let image = egui::ColorImage::new([tex_width, 1], pixels);
        let tex_name = format!("hue_bar_{}", id_salt);
        state.hue_texture =
            Some(ui.ctx().load_texture(tex_name, image, egui::TextureOptions::LINEAR));
    }

    // Paint the texture
    if let Some(tex) = &state.hue_texture {
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter()
            .image(tex.id(), rect, uv, egui::Color32::WHITE);
    }

    // Indicator line at current hue
    let hx = rect.left() + state.hue * rect.width();
    ui.painter().line_segment(
        [egui::pos2(hx, rect.top()), egui::pos2(hx, rect.bottom())],
        egui::Stroke::new(2.0, egui::Color32::WHITE),
    );
    ui.painter().line_segment(
        [egui::pos2(hx, rect.top()), egui::pos2(hx, rect.bottom())],
        egui::Stroke::new(1.0, egui::Color32::BLACK),
    );

    changed
}

// ── Hex input and RGB readout ──────────────────────────────────────

fn draw_hex_and_rgb(
    ui: &mut egui::Ui,
    state: &mut ColorPickerState,
    _id_salt: &str,
) -> bool {
    let mut changed = false;

    // Hex input row
    ui.horizontal(|ui| {
        ui.label("#");
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.hex_input)
                .desired_width(60.0)
                .char_limit(6),
        );
        if response.lost_focus() || response.changed() {
            if let Some(rgb) = parse_hex_color(&state.hex_input) {
                let (h, s, v) = rgb_to_hsv(
                    rgb[0] as f32 / 255.0,
                    rgb[1] as f32 / 255.0,
                    rgb[2] as f32 / 255.0,
                );
                state.hue = h;
                state.sat = s;
                state.val = v;
                state.rgb_input = [
                    rgb[0].to_string(),
                    rgb[1].to_string(),
                    rgb[2].to_string(),
                ];
                state.sv_texture_hue = -1.0;
                changed = true;
            }
        }
    });

    // RGB input row
    ui.horizontal(|ui| {
        let labels = ["R:", "G:", "B:"];
        for i in 0..3 {
            ui.label(labels[i]);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut state.rgb_input[i])
                    .desired_width(30.0)
                    .char_limit(3),
            );
            if resp.lost_focus() || resp.changed() {
                if let Ok(val) = state.rgb_input[i].parse::<u8>() {
                    let mut rgb = state.get_rgb();
                    rgb[i] = val;
                    let (h, s, v) = rgb_to_hsv(
                        rgb[0] as f32 / 255.0,
                        rgb[1] as f32 / 255.0,
                        rgb[2] as f32 / 255.0,
                    );
                    state.hue = h;
                    state.sat = s;
                    state.val = v;
                    state.hex_input = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
                    state.sv_texture_hue = -1.0;
                    changed = true;
                }
            }
        }
    });

    changed
}

/// Parse a hex color string (with or without leading #).
fn parse_hex_color(s: &str) -> Option<[u8; 3]> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some([r, g, b])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_rgb_roundtrip() {
        // Pure red
        let (r, g, b) = hsv_to_rgb(0.0, 1.0, 1.0);
        assert!((r - 1.0).abs() < 0.01);
        assert!(g.abs() < 0.01);
        assert!(b.abs() < 0.01);

        // Roundtrip
        let (h, s, v) = rgb_to_hsv(r, g, b);
        let (r2, g2, b2) = hsv_to_rgb(h, s, v);
        assert!((r - r2).abs() < 0.01);
        assert!((g - g2).abs() < 0.01);
        assert!((b - b2).abs() < 0.01);
    }

    #[test]
    fn parse_hex_valid() {
        assert_eq!(parse_hex_color("FF0000"), Some([255, 0, 0]));
        assert_eq!(parse_hex_color("#00ff00"), Some([0, 255, 0]));
        assert_eq!(parse_hex_color("0000FF"), Some([0, 0, 255]));
    }

    #[test]
    fn parse_hex_invalid() {
        assert_eq!(parse_hex_color("GGHHII"), None);
        assert_eq!(parse_hex_color("FF00"), None);
        assert_eq!(parse_hex_color(""), None);
    }

    #[test]
    fn state_set_get_rgb() {
        let mut state = ColorPickerState::default();
        state.set_rgb([128, 64, 200]);
        let rgb = state.get_rgb();
        // Allow +/- 1 due to float rounding
        assert!((rgb[0] as i16 - 128).abs() <= 1);
        assert!((rgb[1] as i16 - 64).abs() <= 1);
        assert!((rgb[2] as i16 - 200).abs() <= 1);
    }
}
