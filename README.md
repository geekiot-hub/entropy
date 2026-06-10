# Entropy ⌨

Desktop control app for programmable input devices, built with Rust + egui.

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

### macOS app bundle

Build an app bundle and DMG on macOS:

```bash
scripts/build_macos_app.sh
open dist/macos/Entropy.app
```

The script creates `dist/macos/Entropy.app`, a zipped app bundle, and a DMG
when `hdiutil` is available. For Vial/HID access on macOS, Linux udev rules are
not needed.

WASM build coming soon.

## License

GPL-3.0-or-later
