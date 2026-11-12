//! Tactile Swiss design system tokens (dark chassis + light drafting-paper) and small widgets.

use std::sync::atomic::{AtomicBool, Ordering};

use eframe::egui::{self, Color32, CornerRadius, FontId, Stroke, StrokeKind};

use crate::units::GlucoseRange;

static LIGHT: AtomicBool = AtomicBool::new(false);

pub fn set_light(v: bool) {
    LIGHT.store(v, Ordering::Relaxed);
}

pub fn is_light() -> bool {
    LIGHT.load(Ordering::Relaxed)
}

fn pick(dark: (u8, u8, u8), light: (u8, u8, u8)) -> Color32 {
    let c = if is_light() { light } else { dark };
    Color32::from_rgb(c.0, c.1, c.2)
}

/// Fixed near-black ink for text drawn on bright coloured fills.
pub const INK_ON: Color32 = Color32::from_rgb(0x14, 0x14, 0x13);
/// Brand tangerine used for primary keycaps in both themes.
pub const BRAND: Color32 = Color32::from_rgb(0xFF, 0xA5, 0x3A);

pub fn night() -> Color32 { pick((0x11, 0x11, 0x11), (0xFB, 0xFA, 0xF6)) }
pub fn well() -> Color32 { pick((0x16, 0x16, 0x15), (0xFF, 0xFF, 0xFF)) }
pub fn carbon() -> Color32 { pick((0x2B, 0x2B, 0x2B), (0xE8, 0xE4, 0xD7)) }
pub fn plate() -> Color32 { pick((0x33, 0x33, 0x33), (0xF6, 0xF3, 0xEA)) }
pub fn key_fill() -> Color32 { pick((0x3D, 0x3D, 0x3C), (0xFF, 0xFF, 0xFF)) }
pub fn rule() -> Color32 { pick((0x49, 0x49, 0x49), (0xC9, 0xC3, 0xB0)) }
pub fn rule_soft() -> Color32 { pick((0x2A, 0x2A, 0x29), (0xE5, 0xE0, 0xD1)) }
pub fn paper() -> Color32 { pick((0xFC, 0xFC, 0xFB), (0x1C, 0x1B, 0x19)) }
pub fn ink_dim() -> Color32 { pick((0xA8, 0xA6, 0x9E), (0x58, 0x55, 0x4D)) }
pub fn ink_muted() -> Color32 { pick((0x7E, 0x7C, 0x75), (0x8A, 0x86, 0x76)) }

pub fn nav_bg() -> Color32 { pick((0x1C, 0x1C, 0x1B), (0xF1, 0xED, 0xE0)) }
pub fn sel_bg() -> Color32 { pick((0x3A, 0x36, 0x2E), (0xFF, 0xE6, 0xC2)) }
pub fn hover_bg() -> Color32 { pick((0x30, 0x30, 0x2F), (0xF9, 0xEF, 0xD9)) }
pub fn stripe_bg() -> Color32 { pick((0x2F, 0x2F, 0x2E), (0xF1, 0xED, 0xDF)) }

pub fn tangerine() -> Color32 { pick((0xFF, 0xA5, 0x3A), (0xE0, 0x77, 0x00)) }
pub fn butter() -> Color32 { pick((0xFF, 0xE7, 0x91), (0xC9, 0x97, 0x00)) }
pub fn raspberry() -> Color32 { pick((0xFF, 0x98, 0xCF), (0xD2, 0x39, 0x8F)) }
pub fn lavender() -> Color32 { pick((0xB5, 0xB5, 0xFE), (0x56, 0x54, 0xC8)) }
pub fn cyan() -> Color32 { pick((0xAE, 0xF3, 0xF3), (0x0F, 0x8D, 0xA0)) }

// Glucose semantics: stable across every screen.
pub fn g_very_low() -> Color32 { pick((0xE0, 0x3A, 0x7C), (0xC2, 0x18, 0x5B)) }
pub fn g_low() -> Color32 { pick((0xFF, 0x6F, 0x61), (0xE5, 0x48, 0x3B)) }
pub fn g_ok() -> Color32 { pick((0x9B, 0xDB, 0x4D), (0x5C, 0xA1, 0x1A)) }
pub fn g_high() -> Color32 { pick((0xFF, 0xA5, 0x3A), (0xE8, 0x88, 0x00)) }
pub fn g_very_high() -> Color32 { pick((0xF0, 0x5A, 0x28), (0xD1, 0x3A, 0x0F)) }

pub fn range_color(r: GlucoseRange) -> Color32 {
    match r {
        GlucoseRange::VeryLow => g_very_low(),
        GlucoseRange::Low => g_low(),
        GlucoseRange::InRange => g_ok(),
        GlucoseRange::High => g_high(),
        GlucoseRange::VeryHigh => g_very_high(),
    }
}

/// Identity colours for inferred time bands.
pub fn band_color(i: usize) -> Color32 {
    let dark = [(0x7F, 0xD6, 0xE0), (0xA9, 0xA6, 0xF5), (0xF2, 0xD0, 0x5B), (0xF2, 0x8F, 0xC4), (0x86, 0xD9, 0xA8), (0xF2, 0xA6, 0x7C)];
    let light = [(0x0F, 0x8D, 0xA0), (0x56, 0x54, 0xC8), (0xB8, 0x8A, 0x00), (0xD2, 0x39, 0x8F), (0x22, 0x92, 0x5E), (0xCF, 0x5E, 0x24)];
    pick(dark[i % 6], light[i % 6])
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
    let light = is_light();
    let mut v = if light { egui::Visuals::light() } else { egui::Visuals::dark() };
    v.override_text_color = Some(paper());
    v.panel_fill = carbon();
    v.window_fill = plate();
    v.extreme_bg_color = well();
    v.faint_bg_color = stripe_bg();
    v.window_stroke = Stroke::new(1.0, rule());
    v.window_corner_radius = radius();
    v.menu_corner_radius = radius();
    v.selection.bg_fill = alpha(tangerine(), 70);
    v.selection.stroke = Stroke::new(1.0, tangerine());
    v.hyperlink_color = cyan();
    v.window_shadow = egui::Shadow { offset: [2, 3], blur: 0, spread: 0, color: Color32::from_black_alpha(if light { 50 } else { 120 }) };

    let hov = pick((0x4A, 0x4A, 0x48), (0xFB, 0xF1, 0xDD));
    let act = pick((0x24, 0x24, 0x23), (0xEA, 0xE4, 0xD2));
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = plate();
    w.noninteractive.weak_bg_fill = plate();
    w.noninteractive.bg_stroke = Stroke::new(1.0, rule());
    w.noninteractive.fg_stroke = Stroke::new(1.0, ink_dim());
    w.noninteractive.corner_radius = radius();
    w.inactive.bg_fill = key_fill();
    w.inactive.weak_bg_fill = key_fill();
    w.inactive.bg_stroke = Stroke::new(1.0, rule());
    w.inactive.fg_stroke = Stroke::new(1.0, paper());
    w.inactive.corner_radius = radius();
    w.hovered.bg_fill = hov;
    w.hovered.weak_bg_fill = hov;
    w.hovered.bg_stroke = Stroke::new(1.0, tangerine());
    w.hovered.fg_stroke = Stroke::new(1.0, paper());
    w.hovered.corner_radius = radius();
    w.active.bg_fill = act;
    w.active.weak_bg_fill = act;
    w.active.bg_stroke = Stroke::new(1.0, tangerine());
    w.active.fg_stroke = Stroke::new(1.0, paper());
    w.active.corner_radius = radius();
    w.open.corner_radius = radius();
    ctx.set_visuals(v);

    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.scroll = egui::style::ScrollStyle::solid();
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
    ui.label(egui::RichText::new(text.to_uppercase()).font(mono(10.5)).color(ink_muted()));
}

/// Raised faceplate frame.
pub fn plate_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(plate())
        .stroke(Stroke::new(1.0, rule()))
        .corner_radius(radius())
        .inner_margin(egui::Margin::same(14))
}

/// Mono readout line.
pub fn evidence(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).font(mono(11.0)).color(ink_dim()));
}

/// Keycap button. Returns true when clicked.
pub fn key(ui: &mut egui::Ui, text: &str, primary: bool, enabled: bool) -> bool {
    let (fill, ink) = if primary { (BRAND, INK_ON) } else { (key_fill(), paper()) };
    let btn = egui::Button::new(egui::RichText::new(text).font(mono(11.5)).color(ink).strong())
        .fill(fill)
        .stroke(Stroke::new(1.0, if primary { Color32::from_rgb(0xE0, 0x77, 0x00) } else { rule() }))
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
        .map(|(_, t)| ui.painter().layout_no_wrap(t.to_uppercase(), font.clone(), paper()))
        .collect();
    let total: f32 = galleys.iter().map(|g| g.size().x + 24.0).sum();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(total, 30.0), egui::Sense::hover());
    ui.painter().rect(rect, radius(), well(), Stroke::new(1.0, rule()), StrokeKind::Inside);
    let mut x = rect.left();
    for ((val, text), g) in items.iter().zip(galleys) {
        let w = g.size().x + 24.0;
        let seg = egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(w, rect.height()));
        let resp = ui.interact(seg, ui.id().with(text), egui::Sense::click());
        let selected = *current == *val;
        if selected {
            ui.painter().rect_filled(seg.shrink(2.0), radius(), sel_bg());
            ui.painter().rect_filled(
                egui::Rect::from_min_size(egui::pos2(seg.left() + 2.0, seg.bottom() - 4.0), egui::vec2(w - 4.0, 2.0)),
                0.0,
                tangerine(),
            );
        } else if resp.hovered() {
            ui.painter().rect_filled(seg.shrink(2.0), radius(), hover_bg());
        }
        ui.painter().text(
            seg.center() - egui::vec2(0.0, 1.0),
            egui::Align2::CENTER_CENTER,
            text.to_uppercase(),
            font.clone(),
            if selected { paper() } else { ink_dim() },
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
        ui.label(egui::RichText::new(text.to_uppercase()).font(mono(10.5)).color(ink_dim()));
    });
}
