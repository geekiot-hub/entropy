# Entropy ⌨

Keyboard configurator for Ergohaven keyboards, built with Rust + egui.

## Status

🚧 Early development

## Features (planned)

- [x] Project scaffold
- [ ] HID device detection
- [ ] Visual keyboard layout
- [ ] Layer switching
- [ ] Keycode assignment
- [ ] Save/load to device
- [ ] WASM / web version

## Tech stack

| Component | Library |
|-----------|---------|
| UI | [egui](https://github.com/emilk/egui) + eframe |
| HID | [hidapi](https://github.com/ruabmbua/hidapi-rs) |
| Serialization | serde + serde_json |
| WASM target | wasm-bindgen |

## Build

```bash
cargo run                    # desktop
cargo build --release        # release binary
```

WASM build coming soon.

## License

GPL-2.0-or-later
