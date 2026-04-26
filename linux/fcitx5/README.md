# Entropy Fcitx5 backend

This is the Fcitx5 module backend for Entropy Universal Symbols on Wayland/Linux. It watches only Entropy's reserved transport chords (`F13..F24` with `Shift`, `Ctrl`, `Alt`, and `Ctrl+Alt`) and commits the matching Unicode text through Fcitx5.

Unlike the X11 helper path, this does not use global X grabs or `xdotool`, and unlike `evdev/uinput`, it does not need access to `/dev/input`.

## Build

Install Fcitx5 development headers first. Package names vary by distro, for example:

- Arch: `fcitx5`
- Fedora: `fcitx5-devel`
- Debian/Ubuntu: `libfcitx5core-dev` or distro equivalent

Then:

```sh
./linux/fcitx5/install-user.sh
fcitx5 -r
```

Or manually:

```sh
cmake -S linux/fcitx5 -B build/fcitx5 -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=~/.local
cmake --build build/fcitx5
cmake --install build/fcitx5
fcitx5 -r
```

After restart, enable the **Entropy Universal Symbols** addon/module in Fcitx5 configuration if it is not enabled automatically.

## Behavior

- Consumes only Entropy transport chords
- Commits the same Unicode symbols/Cyrillic letters as Smart Input
- Ignores all ordinary typing
- Works through the Fcitx5 input-method stack, including Wayland sessions supported by Fcitx5
