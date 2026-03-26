use serde::{Deserialize, Serialize};

/// A single key with its assigned keycode per layer.
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
    pub keys: Vec<Key>,
    pub layers: Vec<Vec<u16>>, // layers[layer][key_idx] = keycode
}

impl KeyboardLayout {
    pub fn get_keycode(&self, layer: usize, key_idx: usize) -> u16 {
        self.layers
            .get(layer)
            .and_then(|l| l.get(key_idx))
            .copied()
            .unwrap_or(0) // KC_TRNS = 0x0001, KC_NO = 0x0000
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
}
