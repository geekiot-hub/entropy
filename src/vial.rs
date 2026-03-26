/// Vial protocol implementation over HID.
/// Based on vial-gui Python source: protocol/keyboard_comm.py

use anyhow::{bail, Context, Result};

/// Raw HID packet size (without report ID byte)
const MSG_LEN: usize = 32;

// VIA commands
const CMD_VIA_GET_PROTOCOL_VERSION: u8 = 0x01;
const CMD_VIA_GET_LAYER_COUNT: u8 = 0x11;
const CMD_VIA_KEYMAP_GET_BUFFER: u8 = 0x12;
const CMD_VIA_SET_KEYCODE: u8 = 0x04;
const CMD_VIA_VIAL_PREFIX: u8 = 0xFE;

// Vial sub-commands (used after CMD_VIA_VIAL_PREFIX)
const CMD_VIAL_GET_KEYBOARD_ID: u8 = 0x00;
const CMD_VIAL_GET_SIZE: u8 = 0x01;
const CMD_VIAL_GET_DEFINITION: u8 = 0x02;

const BUFFER_FETCH_CHUNK: usize = 28;

#[cfg(not(target_arch = "wasm32"))]
pub struct VialDevice {
    device: hidapi::HidDevice,
}

#[cfg(not(target_arch = "wasm32"))]
impl VialDevice {
    pub fn open(path: &str) -> Result<Self> {
        let api = hidapi::HidApi::new().context("Failed to init hidapi")?;
        let device = api
            .open_path(&std::ffi::CString::new(path)?)
            .context("Failed to open HID device")?;
        Ok(Self { device })
    }

    /// Send exactly MSG_LEN bytes (with 0x00 report ID prepended), receive MSG_LEN bytes back.
    fn usb_send(&self, data: &[u8]) -> Result<[u8; MSG_LEN]> {
        let mut write_buf = [0u8; MSG_LEN + 1];
        write_buf[0] = 0x00; // report ID
        let n = data.len().min(MSG_LEN);
        write_buf[1..1 + n].copy_from_slice(&data[..n]);

        self.device.write(&write_buf).context("HID write failed")?;

        // Read response — hidapi on Windows returns MSG_LEN bytes (no report ID)
        // on Linux/macOS may include report ID prefix
        let mut read_buf = [0u8; MSG_LEN + 1];
        let bytes_read = self.device
            .read_timeout(&mut read_buf, 2000)
            .context("HID read failed")?;

        if bytes_read == 0 {
            bail!("HID timeout — device did not respond");
        }

        let mut resp = [0u8; MSG_LEN];
        if bytes_read == MSG_LEN + 1 {
            // platform included report ID
            resp.copy_from_slice(&read_buf[1..MSG_LEN + 1]);
        } else {
            let copy = bytes_read.min(MSG_LEN);
            resp[..copy].copy_from_slice(&read_buf[..copy]);
        }
        Ok(resp)
    }

    pub fn get_protocol_version(&self) -> Result<u16> {
        let resp = self.usb_send(&[CMD_VIA_GET_PROTOCOL_VERSION])?;
        // resp[1..3] = big-endian u16
        Ok(u16::from_be_bytes([resp[1], resp[2]]))
    }

    pub fn get_layer_count(&self) -> Result<u8> {
        let resp = self.usb_send(&[CMD_VIA_GET_LAYER_COUNT])?;
        Ok(resp[1])
    }

    /// Returns (vial_protocol: u32, keyboard_id: u64)
    pub fn get_keyboard_id(&self) -> Result<(u32, u64)> {
        let resp = self.usb_send(&[CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_KEYBOARD_ID])?;
        let vial_proto = u32::from_le_bytes([resp[0], resp[1], resp[2], resp[3]]);
        let kb_id = u64::from_le_bytes([resp[4], resp[5], resp[6], resp[7],
                                        resp[8], resp[9], resp[10], resp[11]]);
        Ok((vial_proto, kb_id))
    }

    pub fn get_definition_size(&self) -> Result<u32> {
        let resp = self.usb_send(&[CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_SIZE])?;
        // response: size as little-endian u32 starting at byte 0
        Ok(u32::from_le_bytes([resp[0], resp[1], resp[2], resp[3]]))
    }

    pub fn get_layout_json(&self) -> Result<serde_json::Value> {
        let sz = self.get_definition_size()? as usize;
        if sz == 0 || sz > 2_000_000 {
            bail!("Invalid definition size: {sz}");
        }
        log::info!("Vial definition compressed size: {sz} bytes");

        let mut payload = Vec::with_capacity(sz);
        let mut block: u32 = 0;
        let mut remaining = sz;

        while remaining > 0 {
            let mut cmd = [0u8; MSG_LEN];
            cmd[0] = CMD_VIA_VIAL_PREFIX;
            cmd[1] = CMD_VIAL_GET_DEFINITION;
            cmd[2..6].copy_from_slice(&block.to_le_bytes());
            let resp = self.usb_send(&cmd)?;

            let chunk = remaining.min(MSG_LEN);
            payload.extend_from_slice(&resp[..chunk]);
            remaining -= chunk;
            block += 1;
        }

        // Decompress LZMA
        let decompressed = lzma_rs::lzma_decompress(&mut &payload[..])
            .context("Failed to decompress vial definition (lzma)")?;

        let json_str = std::str::from_utf8(&decompressed)
            .context("Vial definition is not valid UTF-8")?;

        let value: serde_json::Value =
            serde_json::from_str(json_str).context("Failed to parse vial JSON")?;
        Ok(value)
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
            let resp = self.usb_send(&cmd)?;
            // response: [cmd, offset_hi, offset_lo, sz, data[0..sz]]
            keymap[offset..offset + sz].copy_from_slice(&resp[4..4 + sz]);
            offset += sz;
        }

        let mut result = vec![0u16; layers * rows * cols];
        for i in 0..result.len() {
            result[i] = u16::from_be_bytes([keymap[i * 2], keymap[i * 2 + 1]]);
        }
        Ok(result)
    }

    pub fn set_keycode(&self, layer: u8, row: u8, col: u8, keycode: u16) -> Result<()> {
        let [hi, lo] = keycode.to_be_bytes();
        self.usb_send(&[CMD_VIA_SET_KEYCODE, layer, row, col, hi, lo])?;
        Ok(())
    }
}
