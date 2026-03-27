use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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



/// Full keyboard layout with multiple layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardLayout {
    pub name: String,
    pub rows: usize,
    pub cols: usize,
    pub keys: Vec<PhysicalKey>,
    pub layers: Vec<Vec<u16>>, // layers[layer][key_idx] = keycode
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

/// Returns true if this KLE key is an encoder (label position 9 == "e")
fn is_encoder_key(label: &str) -> bool {
    label.lines().nth(9).map(|s| s.trim() == "e").unwrap_or(false)
}

impl KeyboardLayout {
    pub fn get_keycode(&self, layer: usize, key_idx: usize) -> u16 {
        self.layers
            .get(layer)
            .and_then(|l| l.get(key_idx))
            .copied()
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
                    if is_encoder_key(label) {
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

        let num_keys = keys.len();
        Ok(Self {
            name,
            rows,
            cols,
            keys,
            layers: vec![vec![0u16; num_keys]; 4], // 4 layers default
        })
    }
}
