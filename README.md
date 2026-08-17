# ENGINE·SIM (Rust GUI)

Modular **cartoon cutaway engine simulator** written in **Rust** with **egui / eframe**.

Animated 4-stroke kinematics: pistons, rods, crank, valves, cams, intake/exhaust flow, live dyno-style metrics, and editable geometry/timing.

## Features

- Procedural multi-cylinder layouts (single, I4, I6, V-twin, V8, boxer presets)
- Slider-crank piston / rod animation with accurate crank angle
- Valve lift from IVO/IVC/EVO/EVC timing + cam half-speed rotation
- Intake (teal) / exhaust (orange) particle flow synchronized to valves
- Editable bore, stroke, rod ratio, compression, V-angle, valve events, throttle
- Live RPM, torque, power estimates + simple graphs
- Keyboard: `Space` pause, `R` reset, `1`–`7` presets, `↑`/`↓` throttle

## Requirements

- Rust **1.85+** (edition 2021)
- For web: `wasm32-unknown-unknown` target + [trunk](https://trunkrs.dev/)

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Run (native desktop)

```bash
cargo run --release
```

## Run (browser / WASM)

```bash
trunk serve
# open http://127.0.0.1:8080
```

Release WASM:

```bash
trunk build --release
# output in dist/
```

## Project layout

| Path | Role |
|------|------|
| `src/sim.rs` | Engine physics, presets, 4-stroke state |
| `src/render.rs` | Cartoon cutaway painter |
| `src/graphs.rs` | Time series / dyno / timing card |
| `src/app.rs` | UI panels, controls, tick loop |
| `src/theme.rs` | Dark cartoon palette |
| `src/main.rs` | Native + WASM entry |

## License

MIT
