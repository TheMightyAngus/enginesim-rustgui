//! Cartoon cutaway engine painter.

use egui::{pos2, vec2, Color32, Pos2, Sense, Stroke, Ui, Vec2};

use crate::sim::{EngineSim, Stroke};
use crate::theme;

struct Particle {
    pos: Pos2,
    vel: Vec2,
    life: f32,
    intake: bool,
}

pub struct EngineView {
    particles: Vec<Particle>,
    accum: f32,
}

impl Default for EngineView {
    fn default() -> Self {
        Self {
            particles: Vec::with_capacity(128),
            accum: 0.0,
        }
    }
}

impl EngineView {
    pub fn show(&mut self, ui: &mut Ui, sim: &EngineSim, dt: f32) {
        let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
        let rect = resp.rect;
        painter.rect_filled(rect, 6.0, theme::BG);

        let n = sim.config.cylinders.len().max(1);
        let cx = rect.center().x;
        let cy = rect.center().y + 20.0;

        let pitch = (rect.width() / (n as f32 + 1.2)).clamp(52.0, 110.0);
        let bore_r = (pitch * 0.28).clamp(16.0, 36.0);
        let stroke_px = bore_r * 2.2;
        let rod_px = stroke_px * sim.config.rod_ratio * 0.55;

        let crank_y = cy + stroke_px * 0.55;

        let fw = pos2(cx - pitch * (n as f32) * 0.5 - 28.0, crank_y);
        let ang = sim.crank_deg.to_radians();
        painter.circle_filled(fw, 22.0, theme::CRANK);
        painter.circle_stroke(fw, 22.0, Stroke::new(2.0, theme::STEEL_LIGHT));
        let spoke = fw + vec2(ang.cos(), ang.sin()) * 16.0;
        painter.line_segment([fw, spoke], Stroke::new(3.0, theme::ACCENT));

        painter.circle_filled(pos2(cx, crank_y), 10.0, theme::STEEL);
        painter.circle_filled(pos2(cx, crank_y), 4.0, theme::ACCENT);

        for i in 0..n {
            let cyl = &sim.config.cylinders[i];
            let st = &sim.cyl[i];
            let bank = cyl.bank_deg.to_radians();
            let base_x = cx + (i as f32 - (n as f32 - 1.0) * 0.5) * pitch;

            let origin = pos2(base_x + bank.sin() * 8.0, crank_y);
            let axis = vec2(-bank.sin(), -bank.cos());

            let piston_travel = st.piston_norm * stroke_px;
            let journal = origin;
            let half = stroke_px * 0.5;
            let rod_len = rod_px + 10.0;
            let j = journal
                + vec2(
                    (sim.crank_deg.to_radians()).sin()
                        * half
                        * 0.9
                        * (if cyl.bank_deg.abs() > 1.0 { 0.4 } else { 1.0 }),
                    (sim.crank_deg.to_radians()).cos() * half * 0.25,
                );
            let tdc = origin + axis * (rod_len + half);
            let pin = tdc - axis * piston_travel;

            let head = pin - axis * (bore_r * 1.8);
            let block_top = tdc - axis * 4.0;
            let block_bot = origin + axis * 4.0;
            draw_bore(&painter, block_top, block_bot, axis, bore_r);

            painter.line_segment([j, pin], Stroke::new(5.0, theme::ROD));
            painter.circle_filled(j, 5.0, theme::STEEL_LIGHT);
            painter.circle_filled(pin, 4.0, theme::STEEL);

            let p0 = pin - axis * (bore_r * 0.15) + perp(axis) * (bore_r * 0.85);
            let p1 = pin - axis * (bore_r * 0.15) - perp(axis) * (bore_r * 0.85);
            let p2 = pin + axis * (bore_r * 0.55) - perp(axis) * (bore_r * 0.85);
            let p3 = pin + axis * (bore_r * 0.55) + perp(axis) * (bore_r * 0.85);
            painter.add(egui::Shape::convex_polygon(
                vec![p0, p1, p2, p3],
                theme::PISTON,
                Stroke::new(1.5, theme::STEEL_LIGHT),
            ));
            for k in 0..2 {
                let ry = pin + axis * (4.0 + k as f32 * 5.0);
                painter.line_segment(
                    [ry + perp(axis) * bore_r * 0.8, ry - perp(axis) * bore_r * 0.8],
                    Stroke::new(1.5, theme::STEEL),
                );
            }

            if st.combust > 0.05 {
                let a = (st.combust * 200.0) as u8;
                painter.circle_filled(
                    pin - axis * bore_r * 0.9,
                    bore_r * 0.7 * st.combust,
                    Color32::from_rgba_unmultiplied(255, 160, 40, a),
                );
            }

            let valve_base = head;
            draw_valve(
                &painter,
                valve_base - perp(axis) * bore_r * 0.45,
                axis,
                st.intake_lift,
                theme::TEAL,
            );
            draw_valve(
                &painter,
                valve_base + perp(axis) * bore_r * 0.45,
                axis,
                st.exhaust_lift,
                theme::ORANGE,
            );

            let plug = head - axis * 10.0;
            painter.line_segment([head - axis * 2.0, plug], Stroke::new(2.0, theme::SPARK));
            painter.circle_filled(plug, 3.0, theme::SPARK);

            let port_in = head - perp(axis) * (bore_r * 1.4) - axis * 6.0;
            let port_ex = head + perp(axis) * (bore_r * 1.4) - axis * 6.0;
            painter.line_segment(
                [head - perp(axis) * bore_r * 0.5, port_in],
                Stroke::new(4.0, Color32::from_rgba_unmultiplied(56, 196, 180, 100)),
            );
            painter.line_segment(
                [head + perp(axis) * bore_r * 0.5, port_ex],
                Stroke::new(4.0, Color32::from_rgba_unmultiplied(240, 140, 48, 100)),
            );

            let badge = head - axis * 28.0;
            let label = format!("C{} {}", i + 1, st.stroke.label());
            let color = match st.stroke {
                Stroke::Intake => theme::TEAL,
                Stroke::Compression => theme::MUTED,
                Stroke::Power => theme::ACCENT,
                Stroke::Exhaust => theme::ORANGE,
            };
            painter.text(
                badge,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.0),
                color,
            );

            self.accum += dt;
            if self.accum > 0.03 {
                if st.intake_lift > 0.15 {
                    self.particles.push(Particle {
                        pos: port_in,
                        vel: (head - port_in).normalized() * 40.0 + axis * 10.0,
                        life: 0.6,
                        intake: true,
                    });
                }
                if st.exhaust_lift > 0.15 {
                    self.particles.push(Particle {
                        pos: head + perp(axis) * 4.0,
                        vel: (port_ex - head).normalized() * 50.0,
                        life: 0.55,
                        intake: false,
                    });
                }
            }
        }
        if self.accum > 0.03 {
            self.accum = 0.0;
        }

        let mut alive = Vec::with_capacity(self.particles.len());
        for mut p in self.particles.drain(..) {
            p.life -= dt;
            p.pos += p.vel * dt;
            if p.life > 0.0 {
                let a = (p.life * 200.0) as u8;
                let c = if p.intake {
                    Color32::from_rgba_unmultiplied(56, 196, 180, a)
                } else {
                    Color32::from_rgba_unmultiplied(240, 140, 48, a)
                };
                painter.circle_filled(p.pos, 2.5, c);
                alive.push(p);
            }
        }
        self.particles = alive;
        if self.particles.len() > 200 {
            self.particles.drain(0..self.particles.len() - 200);
        }

        painter.text(
            pos2(rect.left() + 12.0, rect.top() + 10.0),
            egui::Align2::LEFT_TOP,
            format!("ENGINE·SIM  ·  {}", sim.config.name),
            egui::FontId::proportional(14.0),
            theme::MUTED,
        );
    }
}

fn perp(v: Vec2) -> Vec2 {
    vec2(-v.y, v.x)
}

fn draw_bore(painter: &egui::Painter, top: Pos2, bot: Pos2, axis: Vec2, r: f32) {
    let p = perp(axis);
    let pts = vec![top + p * r, top - p * r, bot - p * r, bot + p * r];
    painter.add(egui::Shape::convex_polygon(
        pts,
        theme::BLOCK,
        Stroke::new(2.0, theme::STEEL),
    ));
}

fn draw_valve(painter: &egui::Painter, base: Pos2, axis: Vec2, lift: f32, color: Color32) {
    let open = lift * 10.0;
    let face = base + axis * (2.0 + open);
    let stem_end = face - axis * 18.0;
    painter.line_segment([face, stem_end], Stroke::new(2.0, theme::VALVE));
    painter.circle_filled(face, 5.0, color);
    let mid = face - axis * 10.0;
    painter.circle_stroke(
        mid,
        4.0 + (1.0 - lift) * 2.0,
        Stroke::new(1.0, theme::STEEL_LIGHT),
    );
}
