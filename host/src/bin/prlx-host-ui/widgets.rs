use eframe::egui::{self, Align, Color32, Layout, RichText, Stroke};

use crate::palette::UiPalette;

pub(crate) fn info_row(ui: &mut egui::Ui, label: &str, value: &str, palette: &UiPalette) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(13.0).color(palette.subtle_text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Prevent weird wrapping by keeping it single-line when possible
            ui.label(RichText::new(value).size(13.0).color(palette.text));
        });
    });
}

pub(crate) fn status_chip(ui: &mut egui::Ui, text: &str, fill: Color32) {
    let frame = egui::Frame::none()
        .fill(fill)
        .rounding(egui::Rounding::same(999.0))
        .inner_margin(egui::Margin::symmetric(12.0, 6.0));

    frame.show(ui, |ui| {
        ui.label(RichText::new(text).size(13.0).color(Color32::WHITE));
    });

    ui.add_space(4.0);
}

pub(crate) fn daemon_label(connected: bool) -> &'static str {
    if connected {
        "Connected"
    } else {
        "Connecting…"
    }
}

pub(crate) fn primary_button<'a>(label: &'a str, palette: &'a UiPalette) -> egui::Button<'a> {
    egui::Button::new(RichText::new(label).size(14.0).color(Color32::WHITE))
        .fill(palette.accent)
        .min_size(egui::vec2(120.0, 38.0))
}

pub(crate) fn secondary_button<'a>(label: &'a str, palette: &'a UiPalette) -> egui::Button<'a> {
    let (fill, text_color) = if palette.is_dark {
        (Color32::from_rgb(50, 54, 64), Color32::WHITE)
    } else {
        (Color32::from_rgb(233, 236, 244), palette.text)
    };

    egui::Button::new(RichText::new(label).size(14.0).color(text_color))
        .fill(fill)
        .min_size(egui::vec2(110.0, 38.0))
}

pub(crate) fn accent_button<'a>(label: &'a str, palette: &'a UiPalette) -> egui::Button<'a> {
    egui::Button::new(RichText::new(label).size(14.0).color(Color32::WHITE))
        .fill(palette.accent)
        .stroke(Stroke::new(1.0, palette.accent))
        .min_size(egui::vec2(140.0, 38.0))
}

pub(crate) fn ghost_button<'a>(label: &'a str, palette: &'a UiPalette) -> egui::Button<'a> {
    egui::Button::new(RichText::new(label).size(14.0).color(palette.text))
        .stroke(Stroke::new(1.0, palette.card_border))
        .min_size(egui::vec2(110.0, 38.0))
}

pub(crate) fn lerp_color(from: Color32, to: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let r = from.r() as f32 + (to.r() as f32 - from.r() as f32) * t;
    let g = from.g() as f32 + (to.g() as f32 - from.g() as f32) * t;
    let b = from.b() as f32 + (to.b() as f32 - from.b() as f32) * t;
    Color32::from_rgb(r as u8, g as u8, b as u8)
}
