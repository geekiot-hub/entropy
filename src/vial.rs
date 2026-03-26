/// Vial protocol implementation over HID.
/// Reference: https://get.vial.today/docs/vial-protocol.html
/// Reference: vial-gui Python source

use anyhow::{bail, Context, Result};
use std::io::Read;

/// Vial raw HID packet size (without report ID)
const MSG_LEN: usize = 32;

// Vial-specific commands — prefixed with 0xFE
const VIAL_PREFIX: u8 = 0xFE;
const VIAL_GET_KEYBOARD_ID: u8 = 0x00;
const VIAL_GET_SIZE: u8 = 0x01;
const VIAL_GET_DEFINITION: u8 = 0x02;

// VIA dynamic keymap commands (no prefix)
const DYNAMIC_KEYMAP_GET_KEYCODE: u8 = 0x04;
const DYNAMIC_KEYMAP_SET_KEYCODE: u8 = 0x05;
const DYNAMIC_KEYMAP_GET_LAYER_COUNT: u8 = 0x11;

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

    /// Send MSG_LEN bytes (prepend report ID 0x00), read back MSG_LEN bytes.
    fn send_raw(&self, msg: &[u8; MSG_LEN]) -> Result<[u8; MSG_LEN]> {
        // Write: [report_id=0x00] + [MSG_LEN bytes]
        let mut write_buf = [0u8; MSG_LEN + 1];
        write_buf[0] = 0x00; // report ID
        write_buf[1..].copy_from_slice(msg);
        self.device.write(&write_buf).context("HID write failed")?;

        // Read back MSG_LEN bytes (hidapi may or may not include report ID)
        let mut read_buf = [0u8; MSG_LEN + 1];
        let n = self
            .device
            .read_timeout(&mut read_buf, 2000)
            .context("HID read failed")?;

        if n == 0 {
            bail!("HID read timeout — device did not respond");
        }

        // Normalize: if read_buf[0] looks like a report ID (0x00) and we got MSG_LEN+1 bytes,
        // skip first byte; otherwise use as-is.
        let mut resp = [0u8; MSG_LEN];
        if n == MSG_LEN + 1 {
            resp.copy_from_slice(&read_buf[1..MSG_LEN + 1]);
        } else {
            let copy = n.min(MSG_LEN);
            resp[..copy].copy_from_slice(&read_buf[..copy]);
        }
        Ok(resp)
    }

    /// Send a vial-specific command: [0xFE, cmd, args..., 0x00 padding]
    fn vial_cmd(&self, cmd: u8, args: &[u8]) -> Result<[u8; MSG_LEN]> {
        let mut msg = [0u8; MSG_LEN];
        msg[0] = VIAL_PREFIX;
        msg[1] = cmd;
        let copy = args.len().min(MSG_LEN - 2);
        msg[2..2 + copy].copy_from_slice(&args[..copy]);
        self.send_raw(&msg)
    }

    /// Send a VIA dynamic keymap command: [cmd, args..., 0x00 padding]
    fn via_cmd(&self, cmd: u8, args: &[u8]) -> Result<[u8; MSG_LEN]> {
        let mut msg = [0u8; MSG_LEN];
        msg[0] = cmd;
        let copy = args.len().min(MSG_LEN - 1);
        msg[1..1 + copy].copy_from_slice(&args[..copy]);
        self.send_raw(&msg)
    }

    pub fn get_keyboard_id(&self) -> Result<[u8; 8]> {
        let resp = self.vial_cmd(VIAL_GET_KEYBOARD_ID, &[])?;
        // response: [0xFE, 0x00, id[0..8]]
        let mut id = [0u8; 8];
        id.copy_from_slice(&resp[2..10]);
        Ok(id)
    }

    pub fn get_definition_size(&self) -> Result<u32> {
        let resp = self.vial_cmd(VIAL_GET_SIZE, &[])?;
        // response: [0xFE, 0x01, size[0..4] little-endian]
        let size = u32::from_le_bytes([resp[2], resp[3], resp[4], resp[5]]);
        Ok(size)
    }

    pub fn get_layout_json(&self) -> Result<serde_json::Value> {
        let size = self.get_definition_size()? as usize;
        if size == 0 || size > 1_000_000 {
            bail!("Invalid definition size: {size}");
        }
        log::info!("Vial definition size: {size} bytes (compressed)");

        let mut compressed = Vec::with_capacity(size);
        let mut page: u16 = 0;

        while compressed.len() < size {
            let resp = self.vial_cmd(VIAL_GET_DEFINITION, &page.to_le_bytes())?;
            // response: [0xFE, 0x02, data[0..30]]
            let data = &resp[2..];
            let remaining = size - compressed.len();
            let chunk = remaining.min(data.len());
            compressed.extend_from_slice(&data[..chunk]);
            page += 1;
        }

        // Decompress zlib
        let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
        let mut json_str = String::new();
        decoder
            .read_to_string(&mut json_str)
            .context("Failed to decompress vial definition")?;

        let value: serde_json::Value =
            serde_json::from_str(&json_str).context("Failed to parse vial JSON")?;
        Ok(value)
    }

    pub fn get_layer_count(&self) -> Result<u8> {
        let resp = self.via_cmd(DYNAMIC_KEYMAP_GET_LAYER_COUNT, &[])?;
        // response: [0x11, count]
        Ok(resp[1])
    }

    pub fn get_keycode(&self, layer: u8, row: u8, col: u8) -> Result<u16> {
        let resp = self.via_cmd(DYNAMIC_KEYMAP_GET_KEYCODE, &[layer, row, col])?;
        // response: [0x04, layer, row, col, keycode_hi, keycode_lo]
        let keycode = u16::from_be_bytes([resp[4], resp[5]]);
        Ok(keycode)
    }

    pub fn set_keycode(&self, layer: u8, row: u8, col: u8, keycode: u16) -> Result<()> {
        let [hi, lo] = keycode.to_be_bytes();
        self.via_cmd(DYNAMIC_KEYMAP_SET_KEYCODE, &[layer, row, col, hi, lo])?;
        Ok(())
    }
}
