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
        let mut cur_x: f32;
        let mut cur_y: f32 = 0.0;
        let mut key_index: usize = 0;

        for kle_row in keymap {
            let row_items = match kle_row.as_array() {
                Some(arr) => arr,
                None => continue,
            };

            // Each new KLE row resets x to 0, increments y by 1
            cur_x = 0.0;
            let mut next_w: f32 = 1.0;
            let mut next_h: f32 = 1.0;
            let mut next_x_offset: f32 = 0.0;
            let mut next_y_offset: f32 = 0.0;

            for item in row_items {
                if let Some(obj) = item.as_object() {
                    // Properties object — modifies the next key
                    if let Some(w) = obj.get("w").and_then(|v| v.as_f64()) {
                        next_w = w as f32;
                    }
                    if let Some(h) = obj.get("h").and_then(|v| v.as_f64()) {
                        next_h = h as f32;
                    }
                    if let Some(x) = obj.get("x").and_then(|v| v.as_f64()) {
                        next_x_offset = x as f32;
                    }
                    if let Some(y) = obj.get("y").and_then(|v| v.as_f64()) {
                        next_y_offset = y as f32;
                    }
                } else if let Some(label) = item.as_str() {
                    // This is a key
                    cur_x += next_x_offset;
                    let key_y = cur_y + next_y_offset;

                    // Determine row/col from layout array or fallback to sequential
                    let (mat_row, mat_col) = if let Some(la) = &layout_array {
                        if let Some(entry) = la.get(key_index) {
                            let r = entry
                                .get("matrix")
                                .or_else(|| entry.get(0))
                                .and_then(|v| {
                                    if let Some(arr) = v.as_array() {
                                        Some((
                                            arr.get(0).and_then(|x| x.as_u64()).unwrap_or(0) as u8,
                                            arr.get(1).and_then(|x| x.as_u64()).unwrap_or(0) as u8,
                                        ))
                                    } else {
                                        None
                                    }
                                });
                            r.unwrap_or(((key_index / cols) as u8, (key_index % cols) as u8))
                        } else {
                            ((key_index / cols) as u8, (key_index % cols) as u8)
                        }
                    } else {
                        ((key_index / cols) as u8, (key_index % cols) as u8)
                    };

                    // Parse label: vial uses "row,col\nlabel" format
                    let display_label = label
                        .lines()
                        .last()
                        .unwrap_or(label)
                        .to_string();

                    keys.push(PhysicalKey {
                        x: cur_x,
                        y: key_y,
                        w: next_w,
                        h: next_h,
                        row: mat_row,
                        col: mat_col,
                        label: display_label,
                    });

                    cur_x += next_w;
                    key_index += 1;

                    // Reset per-key overrides
                    next_w = 1.0;
                    next_h = 1.0;
                    next_x_offset = 0.0;
                    next_y_offset = 0.0;
                }
            }

            cur_y += 1.0;
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
