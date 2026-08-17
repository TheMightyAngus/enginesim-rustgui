//! Simple time-series and timing card drawings.

use egui::{pos2, Stroke, Ui};

use crate::sim::EngineSim;
use crate::theme;

pub fn draw_rpm_history(ui: &mut Ui, sim: &EngineSim) {
    let h = 72.0;
    let (resp, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), h), egui::Sense::hover());
    let rect = resp.rect;
    painter.rect_filled(rect, 4.0, theme::PANEL);
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0, theme::PANEL_EDGE));

    let data = sim.history_rpm();
    if data.len() < 2 {
        return;
    }
    let min_v = 0.0;
    let max_v = 8000.0;
    let n = data.len();
    let mut pts = Vec::with_capacity(n);
    for (i, v) in data.iter().enumerate() {
        let t = i as f32 / (n - 1) as f32;
        let x = rect.left() + t * rect.width();
        let y = rect.bottom() - ((*v - min_v) / (max_v - min_v)).clamp(0.0, 1.0) * rect.height();
        pts.push(pos2(x, y));
    }
    for w in pts.windows(2) {
        painter.line_segment([w[0], w[1]], Stroke::new(1.5, theme::ACCENT));
    }
    painter.text(
        pos2(rect.left() + 6.0, rect.top() + 4.0),
        egui::Align2::LEFT_TOP,
        "RPM",
        egui::FontId::proportional(11.0),
        theme::MUTED,
    );
}

pub fn draw_power_history(ui: &mut Ui, sim: &EngineSim) {
    let h = 72.0;
    let (resp, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), h), egui::Sense::hover());
    let rect = resp.rect;
    painter.rect_filled(rect, 4.0, theme::PANEL);
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0, theme::PANEL_EDGE));

    let data = sim.history_hp();
    if data.len() < 2 {
        return;
    }
    let max_v = data.iter().cloned().fold(50.0_f32, f32::max).max(50.0);
    let n = data.len();
    let mut pts = Vec::with_capacity(n);
    for (i, v) in data.iter().enumerate() {
        let t = i as f32 / (n - 1) as f32;
        let x = rect.left() + t * rect.width();
        let y = rect.bottom() - (*v / max_v).clamp(0.0, 1.0) * (rect.height() - 4.0) - 2.0;
        pts.push(pos2(x, y));
    }
    for w in pts.windows(2) {
        painter.line_segment([w[0], w[1]], Stroke::new(1.5, theme::TEAL));
    }
    painter.text(
        pos2(rect.left() + 6.0, rect.top() + 4.0),
        egui::Align2::LEFT_TOP,
        "HP",
        egui::FontId::proportional(11.0),
        theme::MUTED,
    );
}

pub fn draw_timing_card(ui: &mut Ui, sim: &EngineSim) {
    let t = &sim.config.timing;
    ui.label(egui::RichText::new("VALVE TIMING").size(11.0).color(theme::MUTED));
    ui.monospace(format!("IVO {:+.0}°  IVC {:+.0}°", t.ivo, t.ivc));
    ui.monospace(format!("EVO {:+.0}°  EVC {:+.0}°", t.evo, t.evc));
    ui.monospace(format!(
        "Lift I {:.0}%  E {:.0}%",
        t.intake_lift * 100.0,
        t.exhaust_lift * 100.0
    ));
}

pub fn draw_tach(ui: &mut Ui, rpm: f32) {
    let size = egui::vec2(ui.available_width().min(160.0), 100.0);
    let (resp, painter) = ui.allocate_painter(size, egui::Sense::hover());
    let rect = resp.rect;
    let c = rect.center() + egui::vec2(0.0, 18.0);
    let r = rect.height() * 0.42;
    painter.circle_stroke(c, r, Stroke::new(3.0, theme::STEEL));
    for i in 0..=8 {
        let a = std::f32::consts::PI * (1.0 + i as f32 / 8.0);
        let i0 = c + egui::vec2(a.cos(), a.sin()) * (r - 4.0);
        let i1 = c + egui::vec2(a.cos(), a.sin()) * (r - 12.0);
        let col = if i >= 7 { theme::ACCENT } else { theme::STEEL_LIGHT };
        painter.line_segment([i0, i1], Stroke::new(2.0, col));
    }
    let n = (rpm / 1000.0).clamp(0.0, 8.0);
    let a = std::f32::consts::PI * (1.0 + n / 8.0);
    let tip = c + egui::vec2(a.cos(), a.sin()) * (r - 16.0);
    painter.line_segment([c, tip], Stroke::new(3.0, theme::ACCENT));
    painter.circle_filled(c, 5.0, theme::ACCENT);
    painter.text(
        c + egui::vec2(0.0, r * 0.55),
        egui::Align2::CENTER_CENTER,
        format!("{:.0}", rpm),
        egui::FontId::proportional(18.0),
        theme::TEXT,
    );
    painter.text(
        c + egui::vec2(0.0, r * 0.75),
        egui::Align2::CENTER_CENTER,
        "RPM",
        egui::FontId::proportional(10.0),
        theme::MUTED,
    );
}
