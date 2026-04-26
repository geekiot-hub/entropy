# Entropy Design Language

## Core style

- Use a centered sleek settings list for Settings/Advanced pages
- Baseline page width: `470px`
- Baseline row height: `54px`
- Show at most 6 settings rows in a viewport when a page needs scrolling
- Keep page title/description positions stable unless explicitly requested
- No cards for settings pages; use thin graphite separators between rows
- Labels on the left, controls anchored to the right edge
- Controls are vertically centered between separators
- Reserve a right gutter for custom scrollbars so they never overlap controls

## Palette

- Accent: muted pastel maroon, not blue/purple/green
- Do not use green as a regular UI accent/state; reserve it only for explicit success status when needed
- Outlines: soft graphite only
- Hover: neutral taupe/maroon fill changes; never darken outlines/contours on hover
- Switches: neutral grayscale active state, muted inactive state, no green/sage tint
- Matrix Tester completed keys use muted maroon/taupe, not green

## Controls

Reusable components live in `src/ui_style.rs`:

- `settings_list_row(...)`
- `settings_list_row_with_tooltip(...)`
- `settings_switch(...)`
- `modern_button(...)`
- `modern_text_field(...)`
- `modern_dropdown_button(...)`
- `paint_floating_scrollbar_handle(...)`

Rules:

- Checkboxes on modernized settings pages become `settings_switch`
- Dropdowns use `modern_dropdown_button`, not default `egui::ComboBox`
- Buttons use `modern_button`
- Text inputs use `modern_text_field`, not default framed `TextEdit`
- Floating scrollbar shows only the handle, no track/path, unless a page explicitly needs a track
- Final modern Settings/Advanced code should not keep page-local painters/helpers for standard controls; reusable controls belong in `src/ui_style.rs`
- Page-local control helpers are acceptable only as temporary prototypes, not final commits
- Custom popup lists should be compact, scrollable, and use the same graphite outline/surface style as their dropdown button

## Tooltips and copy

- Tooltips must not end with a period
- Every left-side label on modern Settings/Advanced pages must use `settings_list_row_with_tooltip(...)` and have a meaningful tooltip
- Settings row label tooltip hitboxes must be text-sized, not full-row
- Suppress tooltips while actively scrolling or dragging where applicable
- Win/Cmd/Super/GUI-related labels and tooltips must be OS-dependent by target binary:
  - macOS: Cmd / ⌘
  - Windows: Win
  - Linux/other: Super

## Scrolling

- Keep action buttons outside scrollable content; they must not move with list scrolling
- Action buttons (`Clear`, `Undo`, `Save`, etc.) should be fixed below the viewport, centered, and outside the scrolled layout
- Smooth scroll is allowed on simple settings lists, but if microfreezes appear around `TextEdit`, prefer stable row-based scrolling
- Scrollable Settings/Advanced lists should render only visible rows plus at most one overscan row; avoid rendering off-screen rows with `TextEdit`, tooltips, or custom paint
- Scrollbar drag/click should remain direct and responsive
- Modern settings scrollbars use one pattern: reusable floating handle, no track/path, and a reserved right gutter

## Behavior

- Settings that already use immediate writes or autosave should not regain a manual `Save` button unless explicitly requested
- Successful autosave/immediate writes should stay quiet unless the page already needs visible status feedback

## Versioning and release hygiene

- Use two-digit version components only: after `v1.6.99` comes `v1.7.00`, not `v1.6.100`
- One task = one versioned binary, one commit, one push
- Root repo should contain only one current `entropy-vX.Y.Z.exe`
- Build gate: `cargo build --release --target x86_64-pc-windows-gnu`
