use super::hid_protocol::*;
use super::hid_parse::parse_keymap_u16_be;
use super::HidDevice;
use anyhow::{Context, Result};

#[cfg(not(target_arch = "wasm32"))]
impl HidDevice {
    pub fn get_layer_count(&self) -> Result<u8> {
        let resp = self.usb_send(&[CMD_VIA_GET_LAYER_COUNT])?;
        Ok(resp[1])
    }

    /// Read entire keymap buffer at once (faster than per-key requests).
    /// Returns Vec of keycodes indexed by [layer * rows * cols + row * cols + col].
    pub fn get_keymap_buffer(&self, layers: usize, rows: usize, cols: usize) -> Result<Vec<u16>> {
        let total_bytes = layers * rows * cols * 2;
        let mut keymap = vec![0u8; total_bytes];

        let mut offset = 0usize;
        while offset < total_bytes {
            let sz = (total_bytes - offset).min(BUFFER_FETCH_CHUNK);
            // CMD_VIA_KEYMAP_GET_BUFFER, offset (big-endian u16), size (u8)
            let cmd = [
                CMD_VIA_KEYMAP_GET_BUFFER,
                ((offset >> 8) & 0xFF) as u8,
                (offset & 0xFF) as u8,
                sz as u8,
            ];
            let resp = self
                .usb_send(&cmd)
                .with_context(|| format!("failed to read keymap buffer at offset {offset}"))?;
            // response: [cmd, offset_hi, offset_lo, sz, data[0..sz]]
            keymap[offset..offset + sz].copy_from_slice(&resp[4..4 + sz]);
            offset += sz;
        }

        Ok(parse_keymap_u16_be(&keymap))
    }

    pub fn set_keycode(&self, layer: u8, row: u8, col: u8, keycode: u16) -> Result<()> {
        let [hi, lo] = keycode.to_be_bytes();
        self
            .usb_send(&[CMD_VIA_SET_KEYCODE, layer, row, col, hi, lo])
            .with_context(|| format!("failed to set keycode at layer {layer}, row {row}, col {col}"))?;
        Ok(())
    }

    pub fn get_encoder(&self, layer: u8, idx: u8) -> Result<(u16, u16)> {
        let resp = self
            .usb_send(&[CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_ENCODER, layer, idx])
            .with_context(|| format!("failed to read encoder {idx} on layer {layer}"))?;
        if resp.len() < 4 {
            anyhow::bail!("encoder get response too short for layer {layer}, idx {idx}");
        }
        Ok((
            u16::from_be_bytes([resp[0], resp[1]]),
            u16::from_be_bytes([resp[2], resp[3]]),
        ))
    }

    pub fn set_encoder(&self, layer: u8, idx: u8, direction: u8, keycode: u16) -> Result<()> {
        let bytes = keycode.to_be_bytes();
        let _ = self
            .usb_send(&[
                CMD_VIA_VIAL_PREFIX,
                CMD_VIAL_SET_ENCODER,
                layer,
                idx,
                direction,
                bytes[0],
                bytes[1],
            ])
            .with_context(|| {
                format!("failed to set encoder {idx} direction {direction} on layer {layer}")
            })?;
        Ok(())
    }

}
