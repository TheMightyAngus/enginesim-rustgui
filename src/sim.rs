//! 4-stroke engine model: geometry, kinematics, valve timing, simple dyno.

use std::f32::consts::PI;

#[derive(Clone, Debug)]
pub struct ValveTiming {
    pub ivo: f32,
    pub ivc: f32,
    pub evo: f32,
    pub evc: f32,
    pub intake_lift: f32,
    pub exhaust_lift: f32,
}

impl Default for ValveTiming {
    fn default() -> Self {
        Self {
            ivo: -15.0,
            ivc: 40.0,
            evo: 50.0,
            evc: 10.0,
            intake_lift: 1.0,
            exhaust_lift: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cylinder {
    pub fire_deg: f32,
    pub bank_deg: f32,
}

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub name: String,
    pub cylinders: Vec<Cylinder>,
    pub bore_mm: f32,
    pub stroke_mm: f32,
    pub rod_ratio: f32,
    pub compression: f32,
    pub timing: ValveTiming,
    pub v_angle: f32,
}

impl EngineConfig {
    pub fn displacement_cc(&self) -> f32 {
        let area = PI * (self.bore_mm * 0.5).powi(2);
        area * self.stroke_mm * self.cylinders.len() as f32 / 1000.0
    }
    pub fn rod_mm(&self) -> f32 {
        self.stroke_mm * self.rod_ratio
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stroke {
    Intake,
    Compression,
    Power,
    Exhaust,
}

impl Stroke {
    pub fn label(self) -> &'static str {
        match self {
            Stroke::Intake => "INTAKE",
            Stroke::Compression => "COMPRESS",
            Stroke::Power => "POWER",
            Stroke::Exhaust => "EXHAUST",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CylinderState {
    pub local_deg: f32,
    pub piston_norm: f32,
    pub rod_angle: f32,
    pub intake_lift: f32,
    pub exhaust_lift: f32,
    pub stroke: Stroke,
    pub combust: f32,
}

#[derive(Clone, Debug)]
pub struct EngineSim {
    pub config: EngineConfig,
    pub crank_deg: f32,
    pub rpm: f32,
    pub throttle: f32,
    pub running: bool,
    pub cyl: Vec<CylinderState>,
    pub torque_nm: f32,
    pub power_hp: f32,
    pub fuel_lph: f32,
    pub afr: f32,
    history_rpm: Vec<f32>,
    history_tq: Vec<f32>,
    history_hp: Vec<f32>,
}

fn wrap720(d: f32) -> f32 {
    let mut x = d % 720.0;
    if x < 0.0 {
        x += 720.0;
    }
    x
}

fn valve_lift(local: f32, open: f32, close: f32, max_lift: f32) -> f32 {
    let mut o = open;
    let mut c = close;
    if c < o {
        c += 720.0;
    }
    let mut x = local;
    if x < o {
        x += 720.0;
    }
    if x < o || x > c {
        return 0.0;
    }
    let span = (c - o).max(1.0);
    let t = (x - o) / span;
    let s = 0.5 - 0.5 * (t * 2.0 * PI).cos();
    s * max_lift
}

impl EngineSim {
    pub fn new(config: EngineConfig) -> Self {
        let n = config.cylinders.len();
        let mut s = Self {
            config,
            crank_deg: 0.0,
            rpm: 900.0,
            throttle: 0.35,
            running: true,
            cyl: vec![
                CylinderState {
                    local_deg: 0.0,
                    piston_norm: 0.0,
                    rod_angle: 0.0,
                    intake_lift: 0.0,
                    exhaust_lift: 0.0,
                    stroke: Stroke::Intake,
                    combust: 0.0,
                };
                n
            ],
            torque_nm: 0.0,
            power_hp: 0.0,
            fuel_lph: 0.0,
            afr: 14.7,
            history_rpm: Vec::with_capacity(240),
            history_tq: Vec::with_capacity(240),
            history_hp: Vec::with_capacity(240),
        };
        s.recompute_kinematics();
        s
    }

    pub fn apply_preset(name: &str) -> EngineConfig {
        match name {
            "I4" => inline(4, "Inline-4"),
            "I6" => inline(6, "Inline-6"),
            "V8" => v_engine(8, 90.0, "V8"),
            "Boxer4" => boxer(4, "Boxer-4"),
            "Single" => inline(1, "Single"),
            "I3" => inline(3, "Inline-3"),
            _ => v_engine(2, 60.0, "V-Twin"),
        }
    }

    pub fn reset(&mut self) {
        self.crank_deg = 0.0;
        self.rpm = 900.0;
        self.throttle = 0.35;
        self.running = true;
        self.history_rpm.clear();
        self.history_tq.clear();
        self.history_hp.clear();
        self.recompute_kinematics();
    }

    pub fn set_config(&mut self, config: EngineConfig) {
        let n = config.cylinders.len();
        self.config = config;
        self.cyl.resize(
            n,
            CylinderState {
                local_deg: 0.0,
                piston_norm: 0.0,
                rod_angle: 0.0,
                intake_lift: 0.0,
                exhaust_lift: 0.0,
                stroke: Stroke::Intake,
                combust: 0.0,
            },
        );
        self.recompute_kinematics();
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        if self.running {
            let target = 700.0 + self.throttle * 6200.0;
            let rate = 1.8 + self.throttle * 2.5;
            self.rpm += (target - self.rpm) * (1.0 - (-rate * dt).exp());
            self.rpm = self.rpm.clamp(600.0, 8500.0);
            let deg_per_sec = self.rpm * 6.0;
            self.crank_deg += deg_per_sec * dt;
        }
        self.recompute_kinematics();
        self.recompute_dyno();
        self.push_history();
    }

    fn recompute_kinematics(&mut self) {
        let stroke = self.config.stroke_mm;
        let rod = self.config.rod_mm();
        let half = stroke * 0.5;
        let t = &self.config.timing;

        for (i, cyl) in self.config.cylinders.iter().enumerate() {
            let local = wrap720(self.crank_deg - cyl.fire_deg);
            let theta = local * PI / 180.0;
            let sin = theta.sin();
            let cos = theta.cos();
            let root = (rod * rod - (half * sin).powi(2)).max(0.0).sqrt();
            let from_tdc = half * (1.0 - cos) + (rod - root);
            let piston_norm = (from_tdc / stroke).clamp(0.0, 1.0);
            let rod_angle = (half * sin / rod).asin();

            let in_open = if t.ivo < 0.0 { 720.0 + t.ivo } else { t.ivo };
            let in_close = 180.0 + t.ivc;
            let ex_open = 540.0 - t.evo;
            let ex_close = if t.evc > 0.0 { t.evc } else { 720.0 + t.evc };

            let intake_lift = valve_lift(local, in_open, in_close, t.intake_lift);
            let exhaust_lift = if ex_close < ex_open {
                valve_lift(local, ex_open, 720.0 + ex_close, t.exhaust_lift)
            } else {
                valve_lift(local, ex_open, ex_close, t.exhaust_lift)
            };

            let stroke_phase = match local {
                d if d < 180.0 => Stroke::Intake,
                d if d < 360.0 => Stroke::Compression,
                d if d < 540.0 => Stroke::Power,
                _ => Stroke::Exhaust,
            };

            let combust = if (360.0..420.0).contains(&local) {
                let u = (local - 360.0) / 60.0;
                (1.0 - u) * self.throttle
            } else {
                0.0
            };

            self.cyl[i] = CylinderState {
                local_deg: local,
                piston_norm,
                rod_angle,
                intake_lift,
                exhaust_lift,
                stroke: stroke_phase,
                combust,
            };
        }
    }

    fn recompute_dyno(&mut self) {
        let disp_l = self.config.displacement_cc() / 1000.0;
        let thr = self.throttle;
        let rpm = self.rpm;
        let n = rpm / 1000.0;
        let shape = (-((n - 4.5) / 3.2).powi(2)).exp();
        let mep = 8.5 + thr * 12.0;
        let tq = disp_l * mep * 15.9 * shape * (0.35 + 0.65 * thr);
        self.torque_nm = tq.max(0.0);
        self.power_hp = self.torque_nm * rpm / 7121.0;
        self.fuel_lph = 0.4 + thr * rpm * disp_l * 0.00045;
        self.afr = 12.5 + (1.0 - thr) * 4.0;
    }

    fn push_history(&mut self) {
        const CAP: usize = 240;
        self.history_rpm.push(self.rpm);
        self.history_tq.push(self.torque_nm);
        self.history_hp.push(self.power_hp);
        if self.history_rpm.len() > CAP {
            self.history_rpm.remove(0);
            self.history_tq.remove(0);
            self.history_hp.remove(0);
        }
    }

    pub fn history_rpm(&self) -> &[f32] {
        &self.history_rpm
    }
    pub fn history_tq(&self) -> &[f32] {
        &self.history_tq
    }
    pub fn history_hp(&self) -> &[f32] {
        &self.history_hp
    }
}

fn firing_inline(n: usize) -> Vec<f32> {
    (0..n).map(|i| i as f32 * 720.0 / n as f32).collect()
}

fn inline(n: usize, name: &str) -> EngineConfig {
    let fires = firing_inline(n);
    EngineConfig {
        name: name.into(),
        cylinders: fires
            .into_iter()
            .map(|f| Cylinder {
                fire_deg: f,
                bank_deg: 0.0,
            })
            .collect(),
        bore_mm: 86.0,
        stroke_mm: 86.0,
        rod_ratio: 1.65,
        compression: 10.5,
        timing: ValveTiming::default(),
        v_angle: 0.0,
    }
}

fn v_engine(n: usize, v_angle: f32, name: &str) -> EngineConfig {
    let fires = firing_inline(n);
    let half = n / 2;
    EngineConfig {
        name: name.into(),
        cylinders: fires
            .into_iter()
            .enumerate()
            .map(|(i, f)| {
                let bank = if i < half {
                    -v_angle * 0.5
                } else {
                    v_angle * 0.5
                };
                let fire = if n == 2 && i == 1 { 300.0 } else { f };
                Cylinder {
                    fire_deg: fire,
                    bank_deg: bank,
                }
            })
            .collect(),
        bore_mm: if n == 2 { 96.0 } else { 92.0 },
        stroke_mm: if n == 2 { 66.0 } else { 80.0 },
        rod_ratio: 1.55,
        compression: 11.0,
        timing: ValveTiming::default(),
        v_angle,
    }
}

fn boxer(n: usize, name: &str) -> EngineConfig {
    let fires = firing_inline(n);
    EngineConfig {
        name: name.into(),
        cylinders: fires
            .into_iter()
            .enumerate()
            .map(|(i, f)| Cylinder {
                fire_deg: f,
                bank_deg: if i % 2 == 0 { -90.0 } else { 90.0 },
            })
            .collect(),
        bore_mm: 88.0,
        stroke_mm: 80.0,
        rod_ratio: 1.6,
        compression: 10.0,
        timing: ValveTiming::default(),
        v_angle: 180.0,
    }
}

pub const PRESET_KEYS: [&str; 7] = ["V-Twin", "I4", "I6", "V8", "Boxer4", "Single", "I3"];
