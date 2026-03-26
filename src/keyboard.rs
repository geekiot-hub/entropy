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

/// Extract matrix (row, col) for a key.
/// Vial KLE labels: first line is "row,col", rest is decorative.
/// If layout_array is present, use that instead (it has explicit matrix entries).
fn parse_matrix_pos(
    label: &str,
    key_index: usize,
    cols: usize,
    layout_array: &Option<&Vec<serde_json::Value>>,
) -> (u8, u8) {
    // Try layout_array first (most reliable)
    if let Some(la) = layout_array {
        if let Some(entry) = la.get(key_index) {
            // Entry format: {"label": "...", "matrix": [row, col]} or [row, col]
            if let Some(matrix) = entry.get("matrix").and_then(|v| v.as_array()) {
                let r = matrix.first().and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                let c = matrix.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                return (r, c);
            }
        }
    }

    // Try parsing "row,col" from first line of label
    if let Some(first_line) = label.lines().next() {
        if let Some((r, c)) = first_line.split_once(',') {
            if let (Ok(row), Ok(col)) = (r.trim().parse::<u8>(), c.trim().parse::<u8>()) {
                return (row, col);
            }
        }
    }

    // Fallback: sequential
    ((key_index / cols.max(1)) as u8, (key_index % cols.max(1)) as u8)
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
        // KLE global state — persists across rows
        let mut cur_x: f32 = 0.0;
        let mut cur_y: f32 = 0.0;
        let mut key_index: usize = 0;

        for kle_row in keymap {
            let row_items = match kle_row.as_array() {
                Some(arr) => arr,
                None => continue,
            };

            // Reset x at start of each KLE row; y persists (incremented at end)
            cur_x = 0.0;
            let mut next_w: f32 = 1.0;
            let mut next_h: f32 = 1.0;

            for item in row_items {
                if let Some(obj) = item.as_object() {
                    // Properties object — x/y deltas are cumulative within this row
                    if let Some(w) = obj.get("w").and_then(|v| v.as_f64()) {
                        next_w = w as f32;
                    }
                    if let Some(h) = obj.get("h").and_then(|v| v.as_f64()) {
                        next_h = h as f32;
                    }
                    // x delta: adds to current x (persists for next key placement)
                    if let Some(x) = obj.get("x").and_then(|v| v.as_f64()) {
                        cur_x += x as f32;
                    }
                    // y delta: adds to global y (persists across rows!)
                    if let Some(y) = obj.get("y").and_then(|v| v.as_f64()) {
                        cur_y += y as f32;
                    }
                } else if let Some(label) = item.as_str() {
                    // Determine matrix row/col
                    // In vial KLE, key label format is "row,col\n..." or just use layout array
                    let (mat_row, mat_col) = parse_matrix_pos(label, key_index, cols, &layout_array);

                    // Display label: last non-empty line of the key label
                    let display_label = label
                        .lines()
                        .filter(|l| !l.is_empty())
                        .last()
                        .unwrap_or("")
                        .to_string();

                    keys.push(PhysicalKey {
                        x: cur_x,
                        y: cur_y,
                        w: next_w,
                        h: next_h,
                        row: mat_row,
                        col: mat_col,
                        label: display_label,
                    });

                    cur_x += next_w;
                    key_index += 1;

                    // w/h reset after each key; x/y do NOT reset
                    next_w = 1.0;
                    next_h = 1.0;
                }
            }

            // End of KLE row: advance y by 1
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
