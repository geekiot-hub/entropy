# Entropy IBus backend

Wayland does not allow a normal application to globally intercept keys and inject text. This IBus backend is the safe Wayland-native path for Entropy Universal Symbols: it consumes only the reserved `F13..F24` transport chords and commits the matching Unicode text through IBus.

## Install for current user

```sh
./linux/ibus/install-user.sh
ibus restart
```

Then add/select **Entropy Universal Symbols** in the system input-source/input-method settings.

Required distro packages are usually:

- `ibus`
- `python3-gi`
- `gir1.2-ibus-1.0`

## Behavior

- Handles only Entropy transport keys: `F13..F24` with `Shift`, `Ctrl`, `Alt`, and `Ctrl+Alt`
- Commits the same Unicode symbols as Entropy Smart Input
- Returns `False` for every non-transport key, so normal typing is passed through
- Does not read or log ordinary text input

## Scope

This is the first IBus backend. Fcitx5 can be added later for users who prefer that input method stack.
