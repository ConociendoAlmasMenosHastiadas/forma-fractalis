# egui Slider Sizing

## The Core Problem

Slider track width in egui is NOT controlled by layout allocation. The following approaches all produce identical-looking sliders regardless of the values passed:

- `ui.add_sized([width, height], Slider::new(...))`
- `ui.allocate_ui(Vec2::new(width, height), |ui| ui.add(Slider::new(...)))`
- `ui.scope(|ui| { ui.set_min_width(width); ui.add(Slider::new(...)); })`
- `ui.with_layout(Layout::..., |ui| ui.add(Slider::new(...)))`

These control the _allocated bounding box_, not the slider track itself.

## The Correct API

Slider track length is controlled exclusively by:

```rust
ui.style_mut().spacing.slider_width = desired_px;
```

This is the single source of truth for how wide the track renders.

## Pattern: Full-Width Slider (fills available space)

Save, set, restore around each slider call:

```rust
let orig_w = ui.style().spacing.slider_width;
ui.style_mut().spacing.slider_width = ui.available_width() - 20.0;
if ui.add(egui::Slider::new(&mut value, min..=max)
    .show_value(false)
    .step_by(0.001))
    .changed()
{
    // handle change
}
ui.style_mut().spacing.slider_width = orig_w;
```

The `- 20.0` leaves room for the scrollbar and panel padding. Adjust as needed.

## Pattern: Fixed-Width Slider

```rust
let orig_w = ui.style().spacing.slider_width;
ui.style_mut().spacing.slider_width = 300.0;
ui.add(egui::Slider::new(&mut value, min..=max));
ui.style_mut().spacing.slider_width = orig_w;
```

## Pattern: Taller Slider (via interact_size)

Track height is governed by `spacing.interact_size.y` (default ~18 px):

```rust
let orig_w = ui.style().spacing.slider_width;
let orig_h = ui.style().spacing.interact_size.y;
ui.style_mut().spacing.slider_width = ui.available_width() - 20.0;
ui.style_mut().spacing.interact_size.y = 32.0;
ui.add(egui::Slider::new(&mut value, min..=max).show_value(false));
ui.style_mut().spacing.slider_width = orig_w;
ui.style_mut().spacing.interact_size.y = orig_h;
```

## Where This Is Used

- `gui/src/fractal_gui.rs` - ChaosSymmetry1 a0-a4 parameter sliders (added v0.2.5)

## Context

Discovered during v0.2.5 while trying to make the ChaosSymmetry1 parameter sliders fill the sidebar. Six different layout-based approaches were tested (add_sized, set_min_width, scope, etc.) and all produced the same narrow default-width track. The root cause is that egui's Slider widget reads `style.spacing.slider_width` at render time and ignores the allocated rect width for the track.
