use crate::widgets::secondary_button;
use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Stroke, TextureHandle};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use host::core::logging as loggins;

use crate::daemon::{DaemonCommand, DaemonEvent, DaemonHandle, DaemonStatus, UiState};
use crate::palette::{
    OUTER_MARGIN, UiPalette, card_frame, header_frame, section_header, section_header_colored,
};
use crate::qr::qr_to_image;
use crate::widgets::{
    accent_button, daemon_label, ghost_button, info_row, lerp_color, primary_button, status_chip,
};

// Layout constants (tune here)
const CARD_GAP: f32 = 18.0;
const SECTION_GAP: f32 = 14.0;

pub(crate) struct HostUiApp {
    daemon: DaemonHandle,
    status: DaemonStatus,
    last_error: Option<String>,
    last_warning: Option<String>,
    qr_texture: Option<TextureHandle>,
    qr_payload: Option<String>,
    show_qr_overlay: bool,
    shutdown_rx: Receiver<()>,
    shutdown_initiated: bool,
    dark_mode: bool,
    visuals_mode: Option<bool>,
}

impl HostUiApp {
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        socket_path: PathBuf,
        shutdown_rx: Receiver<()>,
    ) -> Self {
        loggins::info("ui", "HostUiApp::new");

        let daemon = DaemonHandle::new(socket_path);

        let mut app = Self {
            daemon,
            status: DaemonStatus::default(),
            last_error: None,
            last_warning: None,
            qr_texture: None,
            qr_payload: None,
            show_qr_overlay: false,
            shutdown_rx,
            shutdown_initiated: false,
            dark_mode: false,
            visuals_mode: None,
        };

        app.refresh_qr_texture(&cc.egui_ctx);
        app.daemon.send(DaemonCommand::Refresh);
        app
    }

    fn apply_visuals_if_needed(&mut self, ctx: &egui::Context, palette: &UiPalette) {
        if self.visuals_mode == Some(self.dark_mode) {
            return;
        }
        self.visuals_mode = Some(self.dark_mode);

        let mut visuals = if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        // Older-egui safe visuals (no Shadow types).
        // Window/panel fill prevents the “black corners” look when using rounding.
        // If your egui version doesn't have these fields, comment them out.
        visuals.window_fill = palette.background;
        visuals.panel_fill = palette.background;

        // Rounding
        visuals.window_rounding = egui::Rounding::same(16.0);
        visuals.menu_rounding = egui::Rounding::same(12.0);

        // Widget rounding
        visuals.widgets.inactive.rounding = egui::Rounding::same(12.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(12.0);
        visuals.widgets.active.rounding = egui::Rounding::same(12.0);
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(12.0);

        // Strokes
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.card_border);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, palette.card_border);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.accent);

        ctx.set_visuals(visuals);
    }

    fn refresh_qr_texture(&mut self, ctx: &egui::Context) {
        let payload = match &self.status.qr_uri {
            Some(payload) => payload.clone(),
            None => {
                self.qr_texture = None;
                self.qr_payload = None;
                return;
            }
        };

        if self.qr_payload.as_deref() == Some(payload.as_str()) {
            return;
        }

        loggins::debug(
            "ui",
            format!("Refreshing QR texture; payload_len={}", payload.len()),
        );

        if let Some(image) = qr_to_image(&payload, 8) {
            self.qr_texture =
                Some(ctx.load_texture("pairing_qr", image, egui::TextureOptions::NEAREST));
            self.qr_payload = Some(payload);
        } else {
            loggins::warn("ui", "Failed to generate QR image");
        }
    }
}

impl eframe::App for HostUiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Shutdown handling
        if !self.shutdown_initiated {
            match self.shutdown_rx.try_recv() {
                Ok(()) => {
                    loggins::info("ui", "Shutdown requested (signal)");
                    self.shutdown_initiated = true;
                    self.daemon.send(DaemonCommand::Shutdown);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    loggins::warn("ui", "shutdown_rx disconnected");
                    self.shutdown_initiated = true;
                }
            }
        }

        // Drain daemon events
        while let Some(event) = self.daemon.try_recv() {
            match event {
                DaemonEvent::Status(status) => {
                    self.status = status;
                    if self.status.connected {
                        // Clear stale connect errors once the daemon is reachable.
                        self.last_error = None;
                    }
                    self.last_warning = None;
                }
                DaemonEvent::Error(err) => {
                    self.last_error = Some(err);
                }
                DaemonEvent::Warning(warning) => {
                    self.last_warning = Some(warning);
                }
            }
        }

        self.refresh_qr_texture(ctx);

        let palette = if self.dark_mode {
            UiPalette::dark()
        } else {
            UiPalette::light()
        };
        self.apply_visuals_if_needed(ctx, &palette);
        // ---------- HEADER ----------
        egui::TopBottomPanel::top("header")
            .frame(header_frame(&palette))
            .show(ctx, |ui| {
                // Make header tall enough + padded
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    // Left title stack
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Parallax Host")
                                .size(28.0)
                                .strong()
                                .color(palette.text),
                        );
                        ui.label(
                            RichText::new("Local streaming host control panel")
                                .size(13.0)
                                .color(palette.subtle_text),
                        );
                    });

                    // Right controls
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;

                        ui.add(egui::Checkbox::new(
                            &mut self.dark_mode,
                            RichText::new("Dark").size(12.0).color(palette.subtle_text),
                        ));
                    });
                });

                ui.add_space(2.0);
            });

        // ---------- MAIN BODY ----------
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(palette.background))
            .show(ctx, |ui| {
                // Outer padding around everything
                ui.add_space(OUTER_MARGIN);

                // Use a fixed two-column layout that doesn't compress text weirdly
                ui.columns(2, |columns| {
                    let (left_slice, right_slice) = columns.split_at_mut(1);
                    let left = &mut left_slice[0];
                    let right = &mut right_slice[0];

                    left.spacing_mut().item_spacing = egui::vec2(0.0, CARD_GAP);
                    right.spacing_mut().item_spacing = egui::vec2(0.0, CARD_GAP);

                    // LEFT COLUMN
                    card_frame(&palette, palette.card).show(left, |ui| {
                        section_header(ui, "Session", &palette);
                        ui.add_space(SECTION_GAP);

                        info_row(ui, "State", self.status.state.label(), &palette);
                        info_row(ui, "Daemon", daemon_label(self.status.connected), &palette);
                        info_row(ui, "Socket", "Local IPC (Unix socket)", &palette);

                        ui.add_space(SECTION_GAP);

                        let (label, color) = match (self.status.connected, self.status.state) {
                            (false, _) => ("Connecting…", palette.muted),
                            (true, UiState::Streaming) => ("Streaming", palette.accent),
                            (true, UiState::Waiting) => (
                                "Waiting",
                                lerp_color(palette.accent, palette.accent_glow, 0.35),
                            ),
                            (true, _) => (
                                "Ready",
                                lerp_color(palette.accent, palette.accent_glow, 0.25),
                            ),
                        };

                        status_chip(ui, label, color);
                    });

                    card_frame(&palette, palette.card).show(left, |ui| {
                        section_header(ui, "Controls", &palette);
                        ui.add_space(SECTION_GAP);

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;

                            let start = ui.add_enabled(
                                self.status.state != UiState::Streaming,
                                primary_button("▶ Start", &palette),
                            );
                            if start.clicked() {
                                loggins::info("ui", "Start clicked");
                                self.daemon.send(DaemonCommand::StartStream);
                            }

                            let stop = ui.add_enabled(
                                self.status.state == UiState::Streaming,
                                secondary_button("■ Stop", &palette),
                            );
                            if stop.clicked() {
                                loggins::info("ui", "Stop clicked");
                                self.daemon.send(DaemonCommand::StopStream);
                            }

                            if ui.add(ghost_button("↻ Refresh", &palette)).clicked() {
                                loggins::info("ui", "Refresh clicked");
                                self.daemon.send(DaemonCommand::Refresh);
                            }
                        });

                        ui.add_space(SECTION_GAP);
                        ui.label(
                            RichText::new(
                                "Tip: Keep Parallax Host running to allow pairing new clients.",
                            )
                            .size(13.0)
                            .color(palette.subtle_text),
                        );
                    });

                    if let Some(err) = &self.last_error {
                        card_frame(&palette, palette.error_bg).show(left, |ui| {
                            section_header_colored(ui, "Daemon Error", palette.error);
                            ui.add_space(8.0);
                            ui.label(RichText::new(err).size(13.0).color(palette.text));
                        });
                    }

                    if let Some(warning) = &self.last_warning {
                        card_frame(&palette, palette.warning_bg).show(left, |ui| {
                            section_header_colored(ui, "Warning", palette.warning);
                            ui.add_space(8.0);
                            ui.label(RichText::new(warning).size(13.0).color(palette.text));
                        });
                    }

                    // RIGHT COLUMN
                    card_frame(&palette, palette.card).show(right, |ui| {
                        section_header(ui, "Secure Pairing", &palette);
                        ui.add_space(SECTION_GAP);

                        ui.label(
                            RichText::new("Access PIN")
                                .size(13.0)
                                .color(palette.subtle_text),
                        );
                        ui.add_space(6.0);

                        let pin_text = self.status.pin.as_deref().unwrap_or("----");
                        ui.label(
                            RichText::new(pin_text)
                                .size(40.0)
                                .font(FontId::proportional(40.0))
                                .strong()
                                .color(palette.text),
                        );

                        ui.add_space(SECTION_GAP);

                        ui.label(
                            RichText::new("QR Code")
                                .size(13.0)
                                .color(palette.subtle_text),
                        );
                        ui.add_space(8.0);

                        let qr_frame = egui::Frame::none()
                            .fill(palette.qr_bg)
                            .rounding(egui::Rounding::same(18.0))
                            .stroke(Stroke::new(1.0, palette.card_border))
                            .inner_margin(egui::Margin::same(14.0));

                        qr_frame.show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                if let Some(texture) = &self.qr_texture {
                                    // Clamp QR size so it doesn’t explode or compress layout
                                    let mut size = texture.size_vec2();
                                    let max_side = 220.0;
                                    let scale = (max_side / size.x).min(max_side / size.y).min(1.0);
                                    size *= scale;
                                    ui.image((texture.id(), size));
                                } else {
                                    ui.label(
                                        RichText::new("No QR payload from daemon.")
                                            .size(13.0)
                                            .color(palette.subtle_text),
                                    );
                                }
                            });
                        });

                        ui.horizontal(|ui| {
                            ui.add_space(2.0);
                            let view_larger = ui.add_enabled(
                                self.qr_texture.is_some(),
                                accent_button("View larger", &palette),
                            );
                            if view_larger.clicked() {
                                self.show_qr_overlay = true;
                            }
                        });

                        ui.add_space(SECTION_GAP);
                        ui.label(
                            RichText::new(
                                "Open the client app and scan the QR to connect securely.",
                            )
                            .size(13.0)
                            .color(palette.subtle_text),
                        );
                    });
                });

                ui.add_space(OUTER_MARGIN);
            });

        if self.show_qr_overlay && self.qr_texture.is_none() {
            self.show_qr_overlay = false;
        }

        if self.show_qr_overlay {
            let screen_rect = ctx.screen_rect();
            let overlay_padding = 40.0;
            let padded_rect = screen_rect.shrink(overlay_padding);

            egui::Area::new("qr_overlay_backdrop".into())
                .order(egui::Order::Foreground)
                .fixed_pos(screen_rect.min)
                .show(ctx, |ui| {
                    ui.set_min_size(screen_rect.size());
                    let (rect, response) =
                        ui.allocate_exact_size(screen_rect.size(), egui::Sense::click());
                    ui.painter()
                        .rect_filled(rect, 0.0, Color32::from_black_alpha(160));
                    if response.clicked() {
                        self.show_qr_overlay = false;
                    }
                });

            egui::Area::new("qr_overlay_content".into())
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.set_max_size(padded_rect.size());
                    let frame = egui::Frame::none()
                        .fill(palette.card)
                        .rounding(egui::Rounding::same(18.0))
                        .stroke(Stroke::new(1.0, palette.card_border))
                        .inner_margin(egui::Margin::same(20.0));

                    frame.show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("QR Code")
                                        .size(16.0)
                                        .strong()
                                        .color(palette.text),
                                );
                                let close_button = egui::Button::new(
                                    RichText::new("X").size(14.0).color(palette.text),
                                )
                                .fill(palette.qr_bg)
                                .min_size(egui::vec2(32.0, 32.0));
                                if ui.add(close_button).clicked() {
                                    self.show_qr_overlay = false;
                                }
                            });
                            ui.add_space(12.0);
                            if let Some(texture) = &self.qr_texture {
                                let mut size = texture.size_vec2();
                                let max_side = padded_rect.width().min(padded_rect.height()) * 0.7;
                                let scale = (max_side / size.x).min(max_side / size.y);
                                size *= scale;
                                ui.image((texture.id(), size));
                            }
                        });
                    });
                });
        }

        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

impl Drop for HostUiApp {
    fn drop(&mut self) {
        loggins::info("ui", "HostUiApp::drop -> sending Shutdown");
        self.daemon.send(DaemonCommand::Shutdown);
    }
}
