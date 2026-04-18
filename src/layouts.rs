/// Offline embedded keyboard layout database.
///
/// Known Ergohaven layouts are embedded as JSON at compile time.
/// `lookup_layout` matches a ZMK device name to an embedded layout.
use crate::keyboard::{KeyboardLayout, PhysicalKey};
use crate::zmk_proto;

const K03_JSON: &str = include_str!("layouts/k03.json");
const IMPERIAL44_JSON: &str = include_str!("layouts/imperial44.json");
const OP36_JSON: &str = include_str!("layouts/op36.json");

pub struct EmbeddedLayout {
    pub id: &'static str,
    pub name: &'static str,
    pub json: &'static str,
}

static LAYOUTS: &[EmbeddedLayout] = &[
    EmbeddedLayout {
        id: "k03",
        name: "K:03",
        json: K03_JSON,
    },
    EmbeddedLayout {
        id: "imperial44",
        name: "Imperial44",
        json: IMPERIAL44_JSON,
    },
    EmbeddedLayout {
        id: "op36",
        name: "Omega Point 36",
        json: OP36_JSON,
    },
];

/// Try to find an embedded layout matching the device name (case-insensitive substring match).
/// Returns parsed physical keys on success.
pub fn lookup_layout(device_name: &str) -> Option<(&'static EmbeddedLayout, Vec<PhysicalKey>)> {
    // Strip non-alphanumeric for fuzzy match (e.g. "K:03" matches id "k03")
    let dn = device_name.to_lowercase();
    let dn_alnum: String = dn.chars().filter(|c| c.is_alphanumeric()).collect();
    let layout = LAYOUTS.iter().find(|l| {
        let id = l.id.to_lowercase();
        let name = l.name.to_lowercase();
        let id_alnum: String = id.chars().filter(|c| c.is_alphanumeric()).collect();
        let name_alnum: String = name.chars().filter(|c| c.is_alphanumeric()).collect();
        dn.contains(&id)
            || dn.contains(&name)
            || dn_alnum.contains(&id_alnum)
            || dn_alnum.contains(&name_alnum)
    })?;

    let keys = parse_embedded_json(layout.json)?;
    Some((layout, keys))
}

/// Build a KeyboardLayout from ZMK physical layout + keymap data.
/// ZMK coordinates are in centidegrees for rotation and 1/100 KLE units for position.
pub fn build_layout_from_zmk(
    phys: &zmk_proto::keymap::PhysicalLayouts,
    keymap: &zmk_proto::keymap::Keymap,
) -> KeyboardLayout {
    // Use active layout or first available
    let active = phys.active_layout_index as usize;
    let phys_layout = phys.layouts.get(active).or_else(|| phys.layouts.first());

    let (keys, rows, cols) = if let Some(pl) = phys_layout {
        let mut max_row = 0u8;
        let mut max_col = 0u8;
        let mut keys: Vec<PhysicalKey> = pl
            .keys
            .iter()
            .enumerate()
            .map(|(i, k)| {
                // ZMK uses centiunits (1/100 KLE unit)
                let x = k.x as f32 / 100.0;
                let y = k.y as f32 / 100.0;
                let w = if k.width > 0 {
                    k.width as f32 / 100.0
                } else {
                    1.0
                };
                let h = if k.height > 0 {
                    k.height as f32 / 100.0
                } else {
                    1.0
                };
                let r = k.r as f32 / 100.0;
                // ZMK dtsi: rx=0, ry=0 with rotation means "rotate around self" (no extra offset)
                // Apply rotation to get display coordinates, then store with rotation=0
                let (final_x, final_y) = if r != 0.0 {
                    let angle = r.to_radians();
                    let rx = k.rx as f32 / 100.0;
                    let ry = k.ry as f32 / 100.0;
                    // anchor is (rx, ry) — if 0,0 then rotation is around origin which is wrong
                    // treat 0,0 as "rotate around self" = no position change
                    if rx == 0.0 && ry == 0.0 {
                        (x, y) // no rotation offset
                    } else {
                        let dx = x - rx;
                        let dy = y - ry;
                        (
                            rx + dx * angle.cos() - dy * angle.sin(),
                            ry + dx * angle.sin() + dy * angle.cos(),
                        )
                    }
                } else {
                    (x, y)
                };
                // ZMK doesn't have row/col in physical layout — use index
                let row = (i / 12) as u8;
                let col = (i % 12) as u8;
                if row > max_row {
                    max_row = row;
                }
                if col > max_col {
                    max_col = col;
                }
                PhysicalKey {
                    x: final_x,
                    y: final_y,
                    w,
                    h,
                    row,
                    col,
                    label: format!("{i}"),
                    rotation: 0.0,
                    rotation_x: 0.0,
                    rotation_y: 0.0,
                }
            })
            .collect::<Vec<_>>();

        // Symmetrize thumb clusters: equalize average Y of left (r>0) and right (r<0) thumb keys
        // ZMK dtsi has inherent asymmetry in thumb Y coordinates
        {
            // Identify thumb keys by original rotation (non-zero in raw data)
            let left_indices: Vec<usize> = pl
                .keys
                .iter()
                .enumerate()
                .filter(|(_, k)| k.r > 0)
                .map(|(i, _)| i)
                .collect();
            let right_indices: Vec<usize> = pl
                .keys
                .iter()
                .enumerate()
                .filter(|(_, k)| k.r < 0)
                .map(|(i, _)| i)
                .collect();
            if !left_indices.is_empty() && !right_indices.is_empty() {
                let left_avg_y = left_indices.iter().map(|&i| keys[i].y).sum::<f32>()
                    / left_indices.len() as f32;
                let right_avg_y = right_indices.iter().map(|&i| keys[i].y).sum::<f32>()
                    / right_indices.len() as f32;
                let mid_y = (left_avg_y + right_avg_y) / 2.0;
                for &i in &left_indices {
                    keys[i].y += mid_y - left_avg_y;
                }
                for &i in &right_indices {
                    keys[i].y += mid_y - right_avg_y;
                }
            }
        }

        (keys, (max_row + 1) as usize, (max_col + 1) as usize)
    } else {
        // Fallback: try to get key count from keymap
        let key_count = keymap.layers.first().map(|l| l.bindings.len()).unwrap_or(0);
        let cols = 12usize;
        let rows = key_count.div_ceil(cols);
        let keys: Vec<PhysicalKey> = (0..key_count)
            .map(|i| PhysicalKey {
                x: (i % cols) as f32,
                y: (i / cols) as f32,
                w: 1.0,
                h: 1.0,
                row: (i / cols) as u8,
                col: (i % cols) as u8,
                label: format!("{i}"),
                rotation: 0.0,
                rotation_x: 0.0,
                rotation_y: 0.0,
            })
            .collect();
        (keys, rows, cols)
    };

    let name = phys_layout
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "ZMK Keyboard".to_string());
    let num_keys = keys.len();
    let layer_count = keymap.layers.len().max(1);

    KeyboardLayout {
        name,
        rows,
        cols,
        keys,
        encoders: vec![],
        layers: vec![vec![0u16; num_keys]; layer_count],
        encoder_layers: vec![],
        layer_names: vec![],
        custom_keycodes: vec![],
        supports_rgb: false,
        lighting_mode: None,
        firmware: crate::firmware::FirmwareProtocol::Zmk,
        zmk_bindings: vec![],
        zmk_behaviors: vec![],
        zmk_layer_ids: vec![],
        zmk_layer_names: vec![],
    }
}

/// Parse the embedded JSON format: `layouts.default_transform.layout` is an array of
/// `{ row, col, x, y, r? }` with absolute coordinates in KLE units.
fn parse_embedded_json(json: &str) -> Option<Vec<PhysicalKey>> {
    let root: serde_json::Value = serde_json::from_str(json).ok()?;
    let layout_arr = root
        .get("layouts")?
        .get("default_transform")?
        .get("layout")?
        .as_array()?;

    let keys = layout_arr
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let row = entry.get("row").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let col = entry.get("col").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let x = entry.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let y = entry.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let r = entry.get("r").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            // rx/ry default to key position if not specified (KLE convention)
            let rx = entry.get("rx").and_then(|v| v.as_f64()).unwrap_or(x as f64) as f32;
            let ry = entry.get("ry").and_then(|v| v.as_f64()).unwrap_or(y as f64) as f32;

            PhysicalKey {
                x,
                y,
                w: 1.0,
                h: 1.0,
                row,
                col,
                label: format!("{i}"),
                rotation: r,
                rotation_x: rx,
                rotation_y: ry,
            }
        })
        .collect();

    Some(keys)
}
