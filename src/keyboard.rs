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
}

/// A single key with its assigned keycode per layer (legacy, kept for compat).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Key {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub label: String,
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

        // KLE global cursor state (persists across rows)
        let mut cur_x: f32 = 0.0;
        let mut cur_y: f32 = 0.0;
        let mut rotation_x: f32 = 0.0;
        let mut rotation_y: f32 = 0.0;

        for kle_row in keymap {
            let row_items = match kle_row.as_array() {
                Some(arr) => arr,
                None => continue,
            };

            let mut next_w: f32 = 1.0;
            let mut next_h: f32 = 1.0;
            // Did this row reset the cursor via rx/ry?
            let mut has_anchor = false;

            for item in row_items {
                if let Some(obj) = item.as_object() {
                    // rx/ry: absolute anchor — process atomically
                    let new_rx = obj.get("rx").and_then(|v| v.as_f64()).map(|v| v as f32);
                    let new_ry = obj.get("ry").and_then(|v| v.as_f64()).map(|v| v as f32);
                    if new_rx.is_some() || new_ry.is_some() {
                        if let Some(rx) = new_rx { rotation_x = rx; }
                        if let Some(ry) = new_ry { rotation_y = ry; }
                        cur_x = rotation_x;
                        cur_y = rotation_y;
                        has_anchor = true;
                    }

                    // x/y: deltas on current cursor
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
                    // Skip encoder pseudo-keys
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
                    });

                    cur_x += next_w;
                    next_w = 1.0;
                    next_h = 1.0;
                }
            }

            // End of KLE row: if no anchor was set, advance y by 1 and reset x to rotation anchor
            if !has_anchor {
                cur_y += 1.0;
                cur_x = rotation_x; // reset to last rx, NOT zero
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
