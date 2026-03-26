/// Vial protocol implementation over HID.
/// Reference: https://get.vial.today/docs/vial-protocol.html

use anyhow::{bail, Context, Result};
use std::io::Read;

const MSG_LEN: usize = 32;
const RAW_HID_BUFFER_SIZE: usize = MSG_LEN + 1; // +1 for report ID

const VIAL_GET_KEYBOARD_ID: u8 = 0x00;
const VIAL_GET_SIZE: u8 = 0x01;
const VIAL_GET_DEFINITION: u8 = 0x02;

// dynamic_keymap commands (via vial)
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

    fn send_raw(&self, msg: &[u8]) -> Result<Vec<u8>> {
        let mut buf = [0u8; RAW_HID_BUFFER_SIZE];
        // buf[0] = 0x00 report ID
        let copy_len = msg.len().min(MSG_LEN);
        buf[1..1 + copy_len].copy_from_slice(&msg[..copy_len]);

        self.device
            .write(&buf)
            .context("HID write failed")?;

        let mut resp = vec![0u8; RAW_HID_BUFFER_SIZE];
        let n = self
            .device
            .read_timeout(&mut resp, 1000)
            .context("HID read failed")?;
        if n == 0 {
            bail!("HID read timeout");
        }
        // Some platforms include report ID in response, some don't.
        // Normalize to MSG_LEN bytes.
        if resp.len() > MSG_LEN {
            resp = resp[..MSG_LEN].to_vec();
        }
        Ok(resp)
    }

    fn vial_cmd(&self, cmd: u8, args: &[u8]) -> Result<Vec<u8>> {
        let mut msg = vec![0xFE; MSG_LEN]; // vial magic prefix
        msg[0] = cmd;
        let copy_len = args.len().min(MSG_LEN - 1);
        msg[1..1 + copy_len].copy_from_slice(&args[..copy_len]);
        self.send_raw(&msg)
    }

    fn dynamic_keymap_cmd(&self, cmd: u8, args: &[u8]) -> Result<Vec<u8>> {
        let mut msg = vec![0x00; MSG_LEN];
        msg[0] = cmd;
        let copy_len = args.len().min(MSG_LEN - 1);
        msg[1..1 + copy_len].copy_from_slice(&args[..copy_len]);
        self.send_raw(&msg)
    }

    pub fn get_keyboard_id(&self) -> Result<[u8; 8]> {
        let resp = self.vial_cmd(VIAL_GET_KEYBOARD_ID, &[])?;
        let mut id = [0u8; 8];
        id.copy_from_slice(&resp[0..8]);
        Ok(id)
    }

    pub fn get_definition_size(&self) -> Result<u32> {
        let resp = self.vial_cmd(VIAL_GET_SIZE, &[])?;
        let size = u32::from_le_bytes([resp[0], resp[1], resp[2], resp[3]]);
        Ok(size)
    }

    pub fn get_layout_json(&self) -> Result<serde_json::Value> {
        let size = self.get_definition_size()? as usize;
        log::info!("Vial definition size: {} bytes (compressed)", size);

        let mut compressed = Vec::with_capacity(size);
        let mut offset: u32 = 0;
        while (offset as usize) < size {
            let page = (offset / MSG_LEN as u32) as u16;
            let resp = self.vial_cmd(
                VIAL_GET_DEFINITION,
                &page.to_le_bytes(),
            )?;
            let remaining = size - offset as usize;
            let chunk = remaining.min(MSG_LEN);
            compressed.extend_from_slice(&resp[..chunk]);
            offset += chunk as u32;
        }
        compressed.truncate(size);

        // Decompress (zlib/deflate)
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
        let resp = self.dynamic_keymap_cmd(DYNAMIC_KEYMAP_GET_LAYER_COUNT, &[])?;
        Ok(resp[1])
    }

    pub fn get_keycode(&self, layer: u8, row: u8, col: u8) -> Result<u16> {
        let resp =
            self.dynamic_keymap_cmd(DYNAMIC_KEYMAP_GET_KEYCODE, &[layer, row, col])?;
        let keycode = u16::from_be_bytes([resp[4], resp[5]]);
        Ok(keycode)
    }

    pub fn set_keycode(&self, layer: u8, row: u8, col: u8, keycode: u16) -> Result<()> {
        let kc_bytes = keycode.to_be_bytes();
        self.dynamic_keymap_cmd(
            DYNAMIC_KEYMAP_SET_KEYCODE,
            &[layer, row, col, kc_bytes[0], kc_bytes[1]],
        )?;
        Ok(())
    }
}
