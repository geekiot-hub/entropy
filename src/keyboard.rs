use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::firmware::FirmwareProtocol;
use crate::zmk::{BehaviorInfo, ZmkBinding};

/// A physical key on the keyboard with position and matrix mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalKey {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub row: u8,
    pub col: u8,
    pub label: String,
    /// Rotation angle in degrees (for drawing key shape)
    pub rotation: f32,
    /// Rotation anchor X (in KLE units)
    pub rotation_x: f32,
    /// Rotation anchor Y (in KLE units)
    pub rotation_y: f32,
}

/// A visual encoder slot on the keyboard layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalEncoder {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub label: String,
    pub encoder_idx: u8,
    pub direction: u8,
    /// Rotation angle in degrees (for drawing key shape)
    pub rotation: f32,
    /// Rotation anchor X (in KLE units)
    pub rotation_x: f32,
    /// Rotation anchor Y (in KLE units)
    pub rotation_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomKeycode {
    pub name: String,
    pub label: String,
    pub title: String,
}

/// Full keyboard layout with multiple layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardLayout {
    pub name: String,
    pub rows: usize,
    pub cols: usize,
    pub keys: Vec<PhysicalKey>,
    pub encoders: Vec<PhysicalEncoder>,
    pub layers: Vec<Vec<u16>>, // layers[layer][key_idx] = keycode (Vial)
    pub encoder_layers: Vec<Vec<u16>>, // encoder_layers[layer][encoder_visual_idx] = keycode (Vial)
    /// Layer names from descriptor/firmware when available.
    #[serde(default)]
    pub layer_names: Vec<String>,
    /// Custom keycodes from vial JSON: symbolic name, short button label, readable tooltip title.
    pub custom_keycodes: Vec<CustomKeycode>,
    /// Whether the keyboard definition exposes a lighting section for RGB/backlight controls
    #[serde(default)]
    pub supports_rgb: bool,
    /// Firmware type
    #[serde(default = "default_firmware")]
    pub firmware: FirmwareProtocol,
    /// ZMK: bindings per layer (used when firmware == Zmk)
    #[serde(skip)]
    pub zmk_bindings: Vec<Vec<ZmkBinding>>,
    /// ZMK: available behaviors fetched from device
    #[serde(skip)]
    pub zmk_behaviors: Vec<BehaviorInfo>,
    /// ZMK: layer IDs as reported by the device
    #[serde(skip)]
    pub zmk_layer_ids: Vec<u32>,
    /// ZMK: layer names from device
    #[serde(skip)]
    pub zmk_layer_names: Vec<String>,
}

fn default_firmware() -> FirmwareProtocol {
    FirmwareProtocol::Vial
}

/// Parse matrix (row, col) from vial KLE key label.
/// Label first line format: "row,col"
fn parse_matrix_from_label(label: &str) -> Option<(u8, u8)> {
    let first_line = label.lines().next()?;
    let (r, c) = first_line.split_once(',')?;
    let row = r.trim().parse::<u8>().ok()?;
    let col = c.trim().parse::<u8>().ok()?;
    Some((row, col))
}

/// Parse encoder metadata from a Vial KLE label.
/// Vial marks encoders with label position 4 == "e" and label 0 == "idx,dir".
fn parse_encoder_from_label(label: &str) -> Option<(u8, u8)> {
    let lines: Vec<&str> = label.lines().collect();
    let is_encoder = lines.get(4).map(|s| s.trim() == "e").unwrap_or(false)
        || lines.get(9).map(|s| s.trim() == "e").unwrap_or(false);
    if !is_encoder {
        return None;
    }
    let first_line = lines.first()?.trim();
    let (idx, dir) = first_line.split_once(',')?;
    Some((idx.trim().parse().ok()?, dir.trim().parse().ok()?))
}

fn parse_layer_names_from_json(json: &serde_json::Value) -> Vec<String> {
    let candidates = [
        json.get("layer_names"),
        json.get("layerNames"),
        json.get("layers"),
        json.get("layouts").and_then(|v| v.get("labels")),
        json.get("layouts").and_then(|v| v.get("layer_names")),
        json.get("layouts").and_then(|v| v.get("layerNames")),
        json.get("vial").and_then(|v| v.get("layer_names")),
        json.get("vial").and_then(|v| v.get("layerNames")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if let Some(arr) = candidate.as_array() {
            let names: Vec<String> = arr
                .iter()
                .filter_map(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.trim().to_string())
                    } else if let Some(inner) = v.as_array() {
                        inner.first().and_then(|x| x.as_str()).map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                })
                .filter(|s| !s.is_empty())
                .collect();
            if !names.is_empty() {
                return names;
            }
        }
    }

    vec![]
}

impl KeyboardLayout {
    pub fn get_keycode(&self, layer: usize, key_idx: usize) -> u16 {
        self.layers
            .get(layer)
            .and_then(|l| l.get(key_idx))
            .copied()
            .unwrap_or(0)
    }

    pub fn get_encoder_keycode(&self, layer: usize, encoder_visual_idx: usize) -> u16 {
        self.encoder_layers
            .get(layer)
            .and_then(|l| l.get(encoder_visual_idx))
            .copied()
            .unwrap_or(0)
    }

    pub fn encoder_count(&self) -> usize {
        self.encoders
            .iter()
            .map(|e| e.encoder_idx as usize + 1)
            .max()
            .unwrap_or(0)
    }

    pub fn set_keycode(&mut self, layer: usize, key_idx: usize, keycode: u16) {
        while self.layers.len() <= layer {
            self.layers.push(vec![0; self.keys.len()]);
        }
        if let Some(layer_data) = self.layers.get_mut(layer) {
            if let Some(slot) = layer_data.get_mut(key_idx) {
                *slot = keycode;
            }
        }
    }

    /// Parse a Vial JSON descriptor into a KeyboardLayout.
    ///
    /// Vial JSON format:
    /// {
    ///   "name": "...",
    ///   "matrix": {"rows": N, "cols": M},
    ///   "layouts": {
    ///     "keymap": [
    ///       [ {obj_or_string}, "label", ... ],  // KLE rows
    ///       ...
    ///     ]
    ///   }
    /// }
    ///
    /// KLE format: rows are arrays. Items are either:
    /// - A JSON object: modifies properties for the NEXT key (x, y, w, h offsets)
    /// - A string: a key label. The key gets current x/y/w/h, then x advances.
    ///
    /// Matrix indices come from the order keys appear: key_index maps to
    /// "layout" array entries which have [row, col] in the vial JSON.
    pub fn from_vial_json(json: &serde_json::Value) -> Result<Self> {
        let name = json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let matrix = json.get("matrix").context("missing 'matrix' field")?;
        let rows = matrix
            .get("rows")
            .and_then(|v| v.as_u64())
            .context("missing matrix.rows")? as usize;
        let cols = matrix
            .get("cols")
            .and_then(|v| v.as_u64())
            .context("missing matrix.cols")? as usize;

        let layouts = json.get("layouts").context("missing 'layouts' field")?;
        let keymap = layouts
            .get("keymap")
            .and_then(|v| v.as_array())
            .context("missing 'layouts.keymap'")?;

        // Optional: "layout" array provides explicit [row, col] per key index
        let layout_array = layouts
            .get("layout")
            .and_then(|v| v.as_array());

        let mut keys = Vec::new();
        let mut encoders = Vec::new();

        // KLE global state
        let mut cur_x: f32 = 0.0;
        let mut cur_y: f32 = 0.0;
        let mut rotation_x: f32 = 0.0;
        let mut rotation_y: f32 = 0.0;
        let mut rotation_angle: f32 = 0.0; // degrees
        let mut first_row = true;

        for kle_row in keymap {
            let row_items = match kle_row.as_array() {
                Some(arr) => arr,
                None => continue,
            };

            // KLE spec: at the start of each row (except first):
            //   - y advances by 1, x resets to rotation_x
            if !first_row {
                cur_y += 1.0;
                cur_x = rotation_x;
            }
            first_row = false;

            let mut next_w: f32 = 1.0;
            let mut next_h: f32 = 1.0;

            for item in row_items {
                if let Some(obj) = item.as_object() {
                    if let Some(rx) = obj.get("rx").and_then(|v| v.as_f64()) {
                        rotation_x = rx as f32;
                        cur_x = rotation_x;
                    }
                    if let Some(ry) = obj.get("ry").and_then(|v| v.as_f64()) {
                        rotation_y = ry as f32;
                        cur_y = rotation_y;
                    }
                    if let Some(r) = obj.get("r").and_then(|v| v.as_f64()) {
                        rotation_angle = r as f32;
                    }
                    if let Some(x) = obj.get("x").and_then(|v| v.as_f64()) {
                        cur_x += x as f32;
                    }
                    if let Some(y) = obj.get("y").and_then(|v| v.as_f64()) {
                        cur_y += y as f32;
                    }
                    if let Some(w) = obj.get("w").and_then(|v| v.as_f64()) {
                        next_w = w as f32;
                    }
                    if let Some(h) = obj.get("h").and_then(|v| v.as_f64()) {
                        next_h = h as f32;
                    }
                } else if let Some(label) = item.as_str() {
                    if let Some((encoder_idx, direction)) = parse_encoder_from_label(label) {
                        encoders.push(PhysicalEncoder {
                            x: cur_x,
                            y: cur_y,
                            w: next_w,
                            h: next_h,
                            label: label.to_string(),
                            encoder_idx,
                            direction,
                            rotation: rotation_angle,
                            rotation_x,
                            rotation_y,
                        });
                        cur_x += next_w;
                        next_w = 1.0;
                        next_h = 1.0;
                        continue;
                    }

                    let (mat_row, mat_col) = parse_matrix_from_label(label)
                        .unwrap_or(((keys.len() / cols.max(1)) as u8, (keys.len() % cols.max(1)) as u8));

                    keys.push(PhysicalKey {
                        x: cur_x,
                        y: cur_y,
                        w: next_w,
                        h: next_h,
                        row: mat_row,
                        col: mat_col,
                        label: format!("{},{}", mat_row, mat_col),
                        rotation: rotation_angle,
                        rotation_x,
                        rotation_y,
                    });

                    cur_x += next_w;
                    next_w = 1.0;
                    next_h = 1.0;
                }
            }
        }

        let layer_names = parse_layer_names_from_json(json);

        // Parse custom keycodes
        let custom_keycodes = if let Some(customs) = json.get("customKeycodes").and_then(|v| v.as_array()) {
            customs.iter().map(|c| {
                let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let title = c.get("title").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                let short_raw = c.get("shortName").and_then(|v| v.as_str()).unwrap_or("");
                let label = if short_raw.is_empty() {
                    name.clone()
                } else {
                    let parts: Vec<&str> = short_raw.lines().filter(|l| !l.is_empty()).collect();
                    match parts.len() {
                        0 => name.clone(),
                        1 => parts[0].to_string(),
                        _ => format!("{}\n{}", parts[0], parts[1..].join(" ")),
                    }
                };
                let title = if title.is_empty() {
                    name.clone()
                } else {
                    title
                };
                CustomKeycode { name, label, title }
            }).collect()
        } else {
            vec![]
        };

        let num_keys = keys.len();
        let supports_rgb = json.get("lighting").is_some();
        Ok(Self {
            name,
            rows,
            cols,
            keys,
            encoders,
            layers: vec![vec![0u16; num_keys]; 4],
            encoder_layers: vec![],
            layer_names,
            custom_keycodes,
            supports_rgb,
            firmware: FirmwareProtocol::Vial,
            zmk_bindings: vec![],
            zmk_behaviors: vec![],
            zmk_layer_ids: vec![],
            zmk_layer_names: vec![],
        })
    }

    /// Returns ZMK binding for (layer, key_idx), or ZmkBinding::none() if missing.
    pub fn get_zmk_binding(&self, layer: usize, key_idx: usize) -> ZmkBinding {
        self.zmk_bindings
            .get(layer)
            .and_then(|l| l.get(key_idx))
            .cloned()
            .unwrap_or_else(ZmkBinding::none)
    }

    /// Sets ZMK binding for (layer, key_idx).
    pub fn set_zmk_binding(&mut self, layer: usize, key_idx: usize, binding: ZmkBinding) {
        while self.zmk_bindings.len() <= layer {
            self.zmk_bindings.push(vec![ZmkBinding::none(); self.keys.len()]);
        }
        if let Some(layer_data) = self.zmk_bindings.get_mut(layer) {
            if let Some(slot) = layer_data.get_mut(key_idx) {
                *slot = binding;
            }
        }
    }
}
