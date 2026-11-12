//! Tactile Swiss design system tokens and small widgets (see docs/superpowers/specs).

use eframe::egui::{self, Color32, CornerRadius, FontId, Stroke, StrokeKind};

use crate::units::GlucoseRange;

pub const NIGHT: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);
pub const WELL: Color32 = Color32::from_rgb(0x16, 0x16, 0x15);
pub const CARBON: Color32 = Color32::from_rgb(0x2B, 0x2B, 0x2B);
pub const PLATE: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
pub const KEY: Color32 = Color32::from_rgb(0x3D, 0x3D, 0x3C);
pub const RULE: Color32 = Color32::from_rgb(0x49, 0x49, 0x49);
pub const RULE_SOFT: Color32 = Color32::from_rgb(0x2A, 0x2A, 0x29);
pub const PAPER: Color32 = Color32::from_rgb(0xFC, 0xFC, 0xFB);
pub const INK_DIM: Color32 = Color32::from_rgb(0xA8, 0xA6, 0x9E);
pub const INK_MUTED: Color32 = Color32::from_rgb(0x7E, 0x7C, 0x75);

pub const TANGERINE: Color32 = Color32::from_rgb(0xFF, 0xA5, 0x3A);
pub const BUTTER: Color32 = Color32::from_rgb(0xFF, 0xE7, 0x91);
pub const RASPBERRY: Color32 = Color32::from_rgb(0xFF, 0x98, 0xCF);
pub const LAVENDER: Color32 = Color32::from_rgb(0xB5, 0xB5, 0xFE);
pub const CYAN: Color32 = Color32::from_rgb(0xAE, 0xF3, 0xF3);

// Glucose semantics: stable across every screen.
pub const G_VERY_LOW: Color32 = Color32::from_rgb(0xE0, 0x3A, 0x7C);
pub const G_LOW: Color32 = Color32::from_rgb(0xFF, 0x6F, 0x61);
pub const G_OK: Color32 = Color32::from_rgb(0x9B, 0xDB, 0x4D);
pub const G_HIGH: Color32 = Color32::from_rgb(0xFF, 0xA5, 0x3A);
pub const G_VERY_HIGH: Color32 = Color32::from_rgb(0xF0, 0x5A, 0x28);

pub fn range_color(r: GlucoseRange) -> Color32 {
    match r {
        GlucoseRange::VeryLow => G_VERY_LOW,
        GlucoseRange::Low => G_LOW,
        GlucoseRange::InRange => G_OK,
        GlucoseRange::High => G_HIGH,
        GlucoseRange::VeryHigh => G_VERY_HIGH,
    }
}

pub fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

pub fn radius() -> CornerRadius {
    CornerRadius::same(2)
}

pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

pub fn sans(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn apply(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.override_text_color = Some(PAPER);
    v.panel_fill = CARBON;
    v.window_fill = PLATE;
    v.extreme_bg_color = WELL;
    v.faint_bg_color = Color32::from_rgb(0x30, 0x30, 0x2F);
    v.window_stroke = Stroke::new(1.0, RULE);
    v.window_corner_radius = radius();
    v.menu_corner_radius = radius();
    v.selection.bg_fill = alpha(TANGERINE, 90);
    v.selection.stroke = Stroke::new(1.0, TANGERINE);
    v.hyperlink_color = CYAN;
    v.window_shadow = egui::Shadow { offset: [2, 3], blur: 0, spread: 0, color: Color32::from_black_alpha(120) };

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = PLATE;
    w.noninteractive.bg_stroke = Stroke::new(1.0, RULE);
    w.noninteractive.corner_radius = radius();
    w.inactive.bg_fill = KEY;
    w.inactive.weak_bg_fill = KEY;
    w.inactive.bg_stroke = Stroke::new(1.0, RULE);
    w.inactive.corner_radius = radius();
    w.hovered.bg_fill = Color32::from_rgb(0x4A, 0x4A, 0x48);
    w.hovered.weak_bg_fill = Color32::from_rgb(0x4A, 0x4A, 0x48);
    w.hovered.bg_stroke = Stroke::new(1.0, TANGERINE);
    w.hovered.corner_radius = radius();
    w.active.bg_fill = Color32::from_rgb(0x24, 0x24, 0x23);
    w.active.weak_bg_fill = Color32::from_rgb(0x24, 0x24, 0x23);
    w.active.bg_stroke = Stroke::new(1.0, TANGERINE);
    w.active.corner_radius = radius();
    w.open.corner_radius = radius();
    ctx.set_visuals(v);

    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
        s.spacing.interact_size.y = 28.0;
        use egui::TextStyle::*;
        s.text_styles.insert(Heading, sans(20.0));
        s.text_styles.insert(Body, sans(14.0));
        s.text_styles.insert(Button, sans(13.0));
        s.text_styles.insert(Small, mono(10.5));
        s.text_styles.insert(Monospace, mono(12.0));
    });
}

/// Mono, uppercase, tracked-looking instrument label.
pub fn label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text.to_uppercase()).font(mono(10.5)).color(INK_MUTED));
}

/// Raised faceplate frame.
pub fn plate() -> egui::Frame {
    egui::Frame::new()
        .fill(PLATE)
        .stroke(Stroke::new(1.0, RULE))
        .corner_radius(radius())
        .inner_margin(egui::Margin::same(14))
}

/// Mono readout line used under conclusions.
pub fn evidence(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).font(mono(11.0)).color(INK_DIM));
}

/// Keycap button. Returns true when clicked.
pub fn key(ui: &mut egui::Ui, text: &str, primary: bool, enabled: bool) -> bool {
    let (fill, ink) = if primary { (TANGERINE, NIGHT) } else { (KEY, PAPER) };
    let btn = egui::Button::new(egui::RichText::new(text).font(mono(11.5)).color(ink).strong())
        .fill(fill)
        .stroke(Stroke::new(1.0, if primary { Color32::from_rgb(0xFF, 0x88, 0x44) } else { RULE }))
        .corner_radius(radius())
        .min_size(egui::vec2(0.0, 30.0));
    ui.add_enabled(enabled, btn).clicked()
}

/// Segmented switch: adjoining rectangular states, selected is lit with an index line.
pub fn segmented<T: PartialEq + Copy>(ui: &mut egui::Ui, current: &mut T, items: &[(T, &str)]) -> bool {
    let mut changed = false;
    let font = mono(11.5);
    let galleys: Vec<_> = items
        .iter()
        .map(|(_, t)| ui.painter().layout_no_wrap(t.to_uppercase(), font.clone(), PAPER))
        .collect();
    let total: f32 = galleys.iter().map(|g| g.size().x + 24.0).sum();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(total, 30.0), egui::Sense::hover());
    ui.painter().rect(rect, radius(), WELL, Stroke::new(1.0, RULE), StrokeKind::Inside);
    let mut x = rect.left();
    for ((val, text), g) in items.iter().zip(galleys) {
        let w = g.size().x + 24.0;
        let seg = egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(w, rect.height()));
        let resp = ui.interact(seg, ui.id().with(text), egui::Sense::click());
        let selected = *current == *val;
        if selected {
            ui.painter().rect_filled(seg.shrink(2.0), radius(), Color32::from_rgb(0x4A, 0x44, 0x38));
            ui.painter().rect_filled(
                egui::Rect::from_min_size(egui::pos2(seg.left() + 2.0, seg.bottom() - 4.0), egui::vec2(w - 4.0, 2.0)),
                0.0,
                TANGERINE,
            );
        } else if resp.hovered() {
            ui.painter().rect_filled(seg.shrink(2.0), radius(), Color32::from_rgb(0x2A, 0x2A, 0x29));
        }
        ui.painter().text(
            seg.center() - egui::vec2(0.0, 1.0),
            egui::Align2::CENTER_CENTER,
            text.to_uppercase(),
            font.clone(),
            if selected { PAPER } else { INK_DIM },
        );
        if resp.clicked() && !selected {
            *current = *val;
            changed = true;
        }
        x += w;
    }
    changed
}

/// Indicator lamp with adjacent label.
pub fn lamp(ui: &mut egui::Ui, color: Color32, text: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let (r, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
        ui.painter().circle_filled(r.center(), 6.0, alpha(color, 50));
        ui.painter().circle_filled(r.center(), 3.5, color);
        ui.label(egui::RichText::new(text.to_uppercase()).font(mono(10.5)).color(INK_DIM));
    });
}
