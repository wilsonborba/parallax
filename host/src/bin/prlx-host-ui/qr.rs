use eframe::egui::{Color32, ColorImage};
use qrcode::QrCode;

pub(crate) fn qr_to_image(payload: &str, scale: usize) -> Option<ColorImage> {
    let code = QrCode::new(payload.as_bytes()).ok()?;
    let width = code.width();
    let image_size = width * scale;
    let mut pixels = vec![Color32::WHITE; image_size * image_size];

    for y in 0..width {
        for x in 0..width {
            let color = if code[(x, y)] == qrcode::Color::Dark {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
            for dy in 0..scale {
                for dx in 0..scale {
                    let idx = (y * scale + dy) * image_size + (x * scale + dx);
                    pixels[idx] = color;
                }
            }
        }
    }

    Some(ColorImage {
        size: [image_size, image_size],
        pixels,
    })
}
