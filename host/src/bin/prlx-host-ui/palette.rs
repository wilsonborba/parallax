use eframe::egui::{self, Color32, RichText};

pub(crate) const OUTER_MARGIN: f32 = 20.0;

pub(crate) struct UiPalette {
    pub(crate) is_dark: bool,
    pub(crate) background: Color32,
    pub(crate) header: Color32,
    pub(crate) card: Color32,
    pub(crate) card_border: Color32,
    pub(crate) qr_bg: Color32,
    pub(crate) text: Color32,
    pub(crate) subtle_text: Color32,
    pub(crate) accent: Color32,
    pub(crate) accent_glow: Color32,
    pub(crate) muted: Color32,
    pub(crate) error: Color32,
    pub(crate) error_bg: Color32,
    pub(crate) warning: Color32,
    pub(crate) warning_bg: Color32,
}

impl UiPalette {
    pub(crate) fn light() -> Self {
        Self {
            is_dark: false,
            background: Color32::from_rgb(245, 246, 250),
            header: Color32::from_rgb(252, 252, 254),
            card: Color32::from_rgb(255, 255, 255),
            card_border: Color32::from_rgb(223, 227, 236),
            qr_bg: Color32::from_rgb(247, 248, 252),
            text: Color32::from_rgb(18, 22, 29),
            subtle_text: Color32::from_rgb(104, 112, 125),

            // Purple accent (not blue)
            accent: Color32::from_rgb(124, 77, 255), // #7C4DFF
            accent_glow: Color32::from_rgb(176, 145, 255), // softer glow

            muted: Color32::from_rgb(210, 214, 222),
            error: Color32::from_rgb(201, 61, 72),
            error_bg: Color32::from_rgb(255, 238, 240),
            warning: Color32::from_rgb(214, 131, 0),
            warning_bg: Color32::from_rgb(255, 246, 230),
        }
    }

    pub(crate) fn dark() -> Self {
        Self {
            is_dark: true,
            background: Color32::from_rgb(14, 16, 21),
            header: Color32::from_rgb(18, 20, 26),
            card: Color32::from_rgb(24, 27, 34),
            card_border: Color32::from_rgb(46, 50, 60),
            qr_bg: Color32::from_rgb(22, 24, 30),
            text: Color32::from_rgb(236, 239, 244),
            subtle_text: Color32::from_rgb(156, 164, 178),
            accent: Color32::from_rgb(124, 77, 255),
            accent_glow: Color32::from_rgb(176, 145, 255),
            muted: Color32::from_rgb(70, 75, 88),
            error: Color32::from_rgb(255, 104, 112),
            error_bg: Color32::from_rgb(54, 24, 28),
            warning: Color32::from_rgb(255, 184, 92),
            warning_bg: Color32::from_rgb(54, 40, 18),
        }
    }
}

pub(crate) fn header_frame(palette: &UiPalette) -> egui::Frame {
    // More breathing room at the top area + consistent padding
    egui::Frame::none()
        .fill(palette.header)
        .inner_margin(egui::Margin::symmetric(OUTER_MARGIN, 16.0))
}

pub(crate) fn card_frame(_palette: &UiPalette, fill: Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(fill)
        .rounding(egui::Rounding::same(18.0))
        .inner_margin(egui::Margin::same(20.0))
}

pub(crate) fn section_header(ui: &mut egui::Ui, title: &str, palette: &UiPalette) {
    ui.label(RichText::new(title).size(16.0).strong().color(palette.text));
}

pub(crate) fn section_header_colored(ui: &mut egui::Ui, title: &str, color: Color32) {
    ui.label(RichText::new(title).size(16.0).strong().color(color));
}
