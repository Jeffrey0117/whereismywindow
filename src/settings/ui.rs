use std::sync::mpsc::Sender;

use eframe::egui;

use crate::config::BorderStyle;
use crate::settings::data::{SettingsData, SettingsMessage};

pub struct SettingsApp {
    draft: SettingsData,
    tx: Sender<SettingsMessage>,
    border_style_index: usize,
}

const BORDER_STYLE_LABELS: [&str; 2] = ["Solid", "Glow"];

fn border_style_to_index(style: BorderStyle) -> usize {
    match style {
        BorderStyle::Solid => 0,
        BorderStyle::Glow => 1,
    }
}

fn index_to_border_style(index: usize) -> BorderStyle {
    match index {
        0 => BorderStyle::Solid,
        _ => BorderStyle::Glow,
    }
}

impl SettingsApp {
    pub fn new(data: SettingsData, tx: Sender<SettingsMessage>) -> Self {
        let border_style_index = border_style_to_index(data.border_style);
        Self {
            draft: data,
            tx,
            border_style_index,
        }
    }
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("WhereIsMyWindow Settings");
                ui.add_space(8.0);

                // -- Border --
                ui.group(|ui| {
                    ui.strong("Border");
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.draft.border_enabled, "Enable border");

                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        ui.color_edit_button_rgb(&mut self.draft.border_color);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Thickness:");
                        ui.add(
                            egui::Slider::new(&mut self.draft.border_thickness, 1.0..=16.0)
                                .suffix(" px"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Style:");
                        egui::ComboBox::from_id_salt("border_style")
                            .selected_text(BORDER_STYLE_LABELS[self.border_style_index])
                            .show_ui(ui, |ui| {
                                for (i, label) in BORDER_STYLE_LABELS.iter().enumerate() {
                                    ui.selectable_value(&mut self.border_style_index, i, *label);
                                }
                            });
                        self.draft.border_style = index_to_border_style(self.border_style_index);
                    });
                });

                ui.add_space(8.0);

                // -- Flash --
                ui.group(|ui| {
                    ui.strong("Flash");
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.draft.flash_enabled, "Flash on monitor change");

                    ui.horizontal(|ui| {
                        ui.label("Duration:");
                        let mut dur = self.draft.flash_duration_ms as f32;
                        ui.add(egui::Slider::new(&mut dur, 50.0..=500.0).suffix(" ms"));
                        self.draft.flash_duration_ms = dur.round() as u32;
                    });

                    ui.horizontal(|ui| {
                        ui.label("Opacity:");
                        ui.add(egui::Slider::new(&mut self.draft.flash_opacity, 0.05..=0.8));
                    });
                });

                ui.add_space(8.0);

                // -- Monitor Indicators --
                ui.group(|ui| {
                    ui.strong("Monitor Indicators");
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.draft.indicator_enabled, "Show monitor badges");
                });

                ui.add_space(8.0);

                // -- General --
                ui.group(|ui| {
                    ui.strong("General");
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.draft.auto_start, "Start with Windows");
                    ui.checkbox(&mut self.draft.reveal_hotkey_enabled, "Reveal hotkey (Ctrl+Shift+F)");
                });

                ui.add_space(16.0);

                // -- Buttons --
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Apply").clicked() {
                            let _ = self.tx.send(SettingsMessage::Apply(self.draft.clone()));
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Cancel").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.tx.send(SettingsMessage::Closed);
    }
}
