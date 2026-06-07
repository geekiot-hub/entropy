# Entropy IBus backend

Wayland does not allow a normal application to globally intercept keys and inject text. This IBus backend is the safe Wayland-native path for Entropy Universal Symbols and Text Expander: it consumes the reserved `F13..F24` transport chords, watches ordinary typing while selected as an input method, and commits matching text through IBus.

## Install for current user

```sh
./linux/ibus/install-user.sh
ibus restart
```

Then add/select Entropy input sources in the system input-source/input-method settings:

- **Entropy Universal Symbols EN** for the `us` layout
- **Entropy Universal Symbols EN UK** for the `gb` layout
- **Entropy Universal Symbols DE** for the `de` layout
- **Entropy Universal Symbols FR** for the `fr` layout
- **Entropy Universal Symbols ES** for the `es` layout
- **Entropy Universal Symbols IT** for the `it` layout
- **Entropy Universal Symbols PT** for the `pt` layout
- **Entropy Universal Symbols BR** for the `br` layout
- **Entropy Universal Symbols RU** for the `ru` layout

Switch between those system input sources to change language while keeping Entropy Text Expander active.

Required distro packages are usually:

- `ibus`
- `python3-gi`
- `gir1.2-ibus-1.0`

## Behavior

- Handles only Entropy transport keys: `F13..F24` with `Shift`, `Ctrl`, `Alt`, and `Ctrl+Alt`
- Commits the same Unicode symbols as Entropy Smart Input
- Loads Text Expander settings from `~/.config/entropy/app_settings.json`
- Loads primary and selected extra rules from `~/.config/entropy/text_expander_rules/`
- Passes normal typing through unless a trigger matches
- On match, swallows the last trigger key, removes the already typed trigger text through IBus surrounding-text APIs, and commits the replacement
- Does not log ordinary text input

## Scope

This is the IBus backend. Fcitx5 currently handles Universal Symbols only.
