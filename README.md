# Entropy

Modern app for programmable keyboards and input devices, built by Ergohaven.

[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Latest beta](https://img.shields.io/badge/latest-v0.1.0--beta.1-lightgrey.svg)](https://github.com/ergohaven/entropy/releases)
[![Platforms](https://img.shields.io/badge/platforms-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey.svg)](#platforms)

Entropy is a desktop app with a modern, minimalist, and intuitive interface for
configuring Vial-compatible programmable input devices: split keyboards, macropads,
trackballs, touchpad modules, and other hardware that exposes keyboard-style
firmware features through HID.

It is designed to feel direct and predictable: connect a device, pick it from the
device list, and work through layout, keycodes, macros, lighting, pointing controls,
and firmware settings from one coherent interface.

## Highlights

- Modern minimalist design that keeps complex device configuration clear,
  predictable, and easy to navigate.
- Full Vial workflow in one desktop app: layouts, keycodes, macros, combos,
  tap dance, key overrides, RGB, pointing controls, and firmware settings.
- Built for more than keyboards: split keyboards, macropads, trackballs,
  touchpads, encoders, displays, and modular input devices.
- Text Expander: create local text shortcuts and expand them from your
  programmable device.
- Universal Symbols: type symbols, typography, arrows, math signs, currency,
  and custom characters from firmware-friendly transport keys.
- Powerful keycode picker with layouts, symbols, modifiers, macros, tap dance,
  custom keycodes, and smart filtering.
- Matrix Tester and Layout Indicator for testing, debugging, and keeping the
  active layer visible while working.
- Multilingual legends, light/dark themes, UI scaling, import/export, tray mode,
  and Linux device-access helpers.

## Features

### Layout Editing

- Visual layout editor for Vial device definitions.
- Layer switching and layer hover preview.
- Key assignment through a dedicated keycode picker.
- Encoder support with optional encoder visibility controls.
- Custom keycode labels from Vial layout JSON.
- Multilingual key legends.
- Shifted number-row symbols such as `!` over `1`.

### Keycode Picker

- Basic keyboard grid with QWERTY, Colemak, Dvorak, Workman, JCUKEN, and Colemak-DH mappings.
- Tabs for symbols, modifiers, special keys, RGB, macros, tap dance, and custom keycodes.
- Universal Symbols support through reserved transport keys.
- Mouse keys and layer/modifier helpers where firmware support is available.
- Firmware-aware filtering so unsupported keycode groups stay out of the way.

### Vial and Firmware Features

Entropy reads the connected device and shows only the sections that are available
for that firmware.

- Combos: input keys, output key, timing, local slot names.
- Macros: editable macro slots, named macros, key insertion.
- Tap Dance: tap/hold/double-tap style actions with named slots.
- Key Overrides: modifier-aware key replacement rules.
- Auto Shift: enable flags and timeout tuning.
- Mouse Keys: pointer and wheel timing/speed settings.
- Tap-Hold and One Shot: QMK timing and behavior settings.
- Grave Escape and Magic settings where exposed by firmware.
- Layer LEDs and RGB controls for supported lighting backends.
- Touchpad settings for supported Ergohaven-style touchpad firmware.
- Module/display presets and live firmware data where available.

### Testing and Companion Views

- Matrix Tester for live switch testing on supported Vial devices.
- Layout Indicator companion window with opacity and pin controls.
- Pressed-key visualization and layer-aware labels.
- Sticky layout view for keeping the current device map visible while using other apps.

### Desktop Quality of Life

- Light and dark themes with configurable accent color.
- UI scaling controls.
- Import/export of app settings.
- System tray/background mode on supported platforms.
- Linux udev rule installer for Vial device access.
- Local Text Expander and Universal Symbols desktop integration.
- Linux IBus backend for Wayland Text Expander and Universal Symbols.
- Linux Fcitx5 backend for Wayland Universal Symbols.

## Platforms

| Platform | Status | Package |
| --- | --- | --- |
| Linux x86_64 | Primary beta target | AppImage |
| Windows x86_64 | Beta target | Portable ZIP |
| macOS | Source/build-script available | App bundle script |

Public beta builds focus on Linux and Windows first. macOS packaging exists in the
repository for source builds.

## Downloads

Beta builds are published on the
[GitHub Releases](https://github.com/ergohaven/entropy/releases) page:

- `entropy-0.1.0-beta.1-linux-x86_64.AppImage`
- `entropy-0.1.0-beta.1-windows-x86_64.zip`
- `SHA256SUMS.txt`

Windows builds are unsigned during beta, so Windows SmartScreen may warn before
launching the app.

## Quick Start

1. Download the build for your platform from GitHub Releases.
2. Connect a Vial-compatible device.
3. On Linux, install Vial udev rules if Entropy cannot open the device.
4. Launch Entropy.
5. Select the device from the top-left device dropdown.
6. Edit layers, keycodes, advanced firmware features, or app settings.
7. Save/write changes when the edited feature requires it.

## Linux Device Access

Vial devices use hidraw access on Linux. If your device appears but cannot be opened,
install the included udev rule:

```sh
./linux/udev/install-vial-rules.sh
```

Replug the device after installing the rule.

## Compatibility

Entropy currently communicates with Vial-compatible HID devices. Its UI is designed
for programmable keyboards and adjacent input devices such as macropads, trackballs,
touchpads, and encoder/display modules when those features are exposed by firmware.

Best-tested hardware is Ergohaven hardware and Vial-compatible QMK/RMK-style devices.
Firmware support varies by device; Entropy hides firmware-gated pages when the
connected device does not expose the required capability.

Not in scope for this beta:

- Browser-only configuration.
- Mobile platforms.

## Development

Install a stable Rust toolchain, then build the desktop app:

```sh
cargo run
cargo build --release
```

Linux builds require native GUI/HID dependencies. On Debian/Ubuntu-like systems:

```sh
sudo apt-get install \
  libhidapi-dev \
  libudev-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libssl-dev \
  libgtk-3-dev
```

Build a macOS app bundle on macOS:

```sh
scripts/build_macos_app.sh
```

Build a Windows release binary from Linux with the GNU target:

```sh
cargo build --release --target x86_64-pc-windows-gnu
```

## Changelog

- [CHANGELOG.md](CHANGELOG.md)

## License

Entropy is licensed under GPL-3.0-or-later. See [LICENSE](LICENSE).
