//! Main application: panels, controls, tick loop.

use egui::{Context, RichText, Slider};

use crate::graphs::{draw_power_history, draw_rpm_history, draw_tach, draw_timing_card};
use crate::render::EngineView;
use crate::sim::{EngineSim, PRESET_KEYS};
use crate::theme;

pub struct EngineApp {
    pub sim: EngineSim,
    view: EngineView,
    show_left: bool,
    show_right: bool,
}

impl Default for EngineApp {
    fn default() -> Self {
        let cfg = EngineSim::apply_preset("V-Twin");
        Self {
            sim: EngineSim::new(cfg),
            view: EngineView::default(),
            show_left: true,
            show_right: true,
        }
    }
}

impl EngineApp {
    fn handle_keys(&mut self, ctx: &Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.sim.running = !self.sim.running;
            }
            if i.key_pressed(egui::Key::R) {
                self.sim.reset();
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                self.sim.throttle = (self.sim.throttle + 0.05).min(1.0);
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                self.sim.throttle = (self.sim.throttle - 0.05).max(0.0);
            }
            for (idx, name) in PRESET_KEYS.iter().enumerate() {
                let key = match idx {
                    0 => egui::Key::Num1,
                    1 => egui::Key::Num2,
                    2 => egui::Key::Num3,
                    3 => egui::Key::Num4,
                    4 => egui::Key::Num5,
                    5 => egui::Key::Num6,
                    6 => egui::Key::Num7,
                    _ => continue,
                };
                if i.key_pressed(key) {
                    self.sim.set_config(EngineSim::apply_preset(name));
                }
            }
        });
    }

    fn left_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.label(RichText::new("ENGINE·SIM").strong().color(theme::ACCENT).size(16.0));
        ui.label(RichText::new("cartoon cutaway").color(theme::MUTED).size(11.0));
        ui.separator();

        ui.label(RichText::new("PRESETS").size(11.0).color(theme::MUTED));
        ui.horizontal_wrapped(|ui| {
            for name in PRESET_KEYS {
                let selected = self.sim.config.name.contains(name) || self.sim.config.name == *name;
                if ui.selectable_label(selected, name).clicked() {
                    self.sim.set_config(EngineSim::apply_preset(name));
                }
            }
        });
        ui.separator();

        ui.label(RichText::new("GEOMETRY").size(11.0).color(theme::MUTED));
        ui.add(Slider::new(&mut self.sim.config.bore_mm, 50.0..=120.0).text("Bore mm"));
        ui.add(Slider::new(&mut self.sim.config.stroke_mm, 40.0..=120.0).text("Stroke mm"));
        ui.add(Slider::new(&mut self.sim.config.rod_ratio, 1.2..=2.0).text("Rod ratio"));
        ui.add(Slider::new(&mut self.sim.config.compression, 7.0..=14.0).text("CR"));
        ui.add(Slider::new(&mut self.sim.config.v_angle, 0.0..=180.0).text("V angle"));

        ui.separator();
        ui.label(RichText::new("VALVE TIMING").size(11.0).color(theme::MUTED));
        let t = &mut self.sim.config.timing;
        ui.add(Slider::new(&mut t.ivo, -40.0..=20.0).text("IVO"));
        ui.add(Slider::new(&mut t.ivc, 0.0..=80.0).text("IVC"));
        ui.add(Slider::new(&mut t.evo, 20.0..=80.0).text("EVO"));
        ui.add(Slider::new(&mut t.evc, -10.0..=40.0).text("EVC"));
        ui.add(Slider::new(&mut t.intake_lift, 0.3..=1.2).text("I lift"));
        ui.add(Slider::new(&mut t.exhaust_lift, 0.3..=1.2).text("E lift"));

        ui.separator();
        ui.label(
            RichText::new(format!(
                "Disp {:.0} cc · {} cyl",
                self.sim.config.displacement_cc(),
                self.sim.config.cylinders.len()
            ))
            .color(theme::MUTED)
            .size(11.0),
        );
        ui.label(
            RichText::new("Keys: Space pause · R reset · 1-7 presets · Up/Down throttle")
                .color(theme::MUTED)
                .size(10.0),
        );
    }

    fn right_panel(&mut self, ui: &mut egui::Ui) {
        draw_tach(ui, self.sim.rpm);
        ui.separator();
        metric(ui, "Torque", self.sim.torque_nm, "Nm");
        metric(ui, "Power", self.sim.power_hp, "HP");
        metric(ui, "Fuel", self.sim.fuel_lph, "L/h");
        metric(ui, "AFR", self.sim.afr, "");
        metric(ui, "Throttle", self.sim.throttle * 100.0, "%");
        ui.separator();
        ui.label(RichText::new("THROTTLE").size(11.0).color(theme::MUTED));
        ui.add(Slider::new(&mut self.sim.throttle, 0.0..=1.0).show_value(false));
        ui.horizontal(|ui| {
            let label = if self.sim.running { "STOP" } else { "PLAY" };
            if ui
                .add(egui::Button::new(RichText::new(label).color(theme::TEXT)).fill(theme::ACCENT_DIM))
                .clicked()
            {
                self.sim.running = !self.sim.running;
            }
            if ui.button("RESET").clicked() {
                self.sim.reset();
            }
        });
        ui.separator();
        draw_rpm_history(ui, &self.sim);
        ui.add_space(6.0);
        draw_power_history(ui, &self.sim);
        ui.add_space(6.0);
        draw_timing_card(ui, &self.sim);

        if !self.sim.cyl.is_empty() {
            ui.separator();
            ui.label(RichText::new("CYLINDERS").size(11.0).color(theme::MUTED));
            for (i, c) in self.sim.cyl.iter().enumerate() {
                ui.monospace(format!(
                    "C{} {:>8}  I{:.0}% E{:.0}%",
                    i + 1,
                    c.stroke.label(),
                    c.intake_lift * 100.0,
                    c.exhaust_lift * 100.0
                ));
            }
        }
    }
}

fn metric(ui: &mut egui::Ui, label: &str, value: f32, unit: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(theme::MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if unit.is_empty() {
                ui.label(RichText::new(format!("{:.1}", value)).strong());
            } else {
                ui.label(RichText::new(format!("{:.0} {}", value, unit)).strong());
            }
        });
    });
}

impl eframe::App for EngineApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt).clamp(0.0, 0.05);
        self.handle_keys(ctx);
        self.sim.tick(dt);

        let narrow = ctx.screen_rect().width() < 900.0;
        if narrow {
            egui::TopBottomPanel::top("top_bar")
                .frame(egui::Frame::none().fill(theme::PANEL).inner_margin(6.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cfg").clicked() {
                            self.show_left = !self.show_left;
                        }
                        ui.label(RichText::new("ENGINE·SIM").strong().color(theme::ACCENT));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Stats").clicked() {
                                self.show_right = !self.show_right;
                            }
                            ui.label(
                                RichText::new(format!("{:.0} RPM", self.sim.rpm)).color(theme::TEXT),
                            );
                        });
                    });
                });
        }

        if self.show_left && !narrow {
            egui::SidePanel::left("left")
                .exact_width(240.0)
                .frame(
                    egui::Frame::none()
                        .fill(theme::PANEL)
                        .stroke(egui::Stroke::new(1.0, theme::PANEL_EDGE))
                        .inner_margin(10.0),
                )
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| self.left_panel(ui));
                });
        } else if self.show_left && narrow {
            egui::SidePanel::left("left_m")
                .exact_width(220.0)
                .frame(
                    egui::Frame::none()
                        .fill(theme::PANEL)
                        .stroke(egui::Stroke::new(1.0, theme::PANEL_EDGE))
                        .inner_margin(8.0),
                )
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| self.left_panel(ui));
                });
        }

        if self.show_right && !narrow {
            egui::SidePanel::right("right")
                .exact_width(200.0)
                .frame(
                    egui::Frame::none()
                        .fill(theme::PANEL)
                        .stroke(egui::Stroke::new(1.0, theme::PANEL_EDGE))
                        .inner_margin(10.0),
                )
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| self.right_panel(ui));
                });
        } else if self.show_right && narrow {
            egui::SidePanel::right("right_m")
                .exact_width(180.0)
                .frame(
                    egui::Frame::none()
                        .fill(theme::PANEL)
                        .stroke(egui::Stroke::new(1.0, theme::PANEL_EDGE))
                        .inner_margin(8.0),
                )
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| self.right_panel(ui));
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(theme::BG).inner_margin(4.0))
            .show(ctx, |ui| {
                self.view.show(ui, &self.sim, dt);
            });

        if self.sim.running {
            ctx.request_repaint();
        }
    }
}
