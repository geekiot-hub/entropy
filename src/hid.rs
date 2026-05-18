/// Vial protocol implementation over HID.
/// Based on vial-gui Python source: protocol/keyboard_comm.py
use anyhow::{bail, Context, Result};

#[path = "hid_protocol.rs"]
mod hid_protocol;
use hid_protocol::*;

#[path = "hid_dynamic.rs"]
mod hid_dynamic;

#[path = "hid_macros.rs"]
mod hid_macros;

#[path = "hid_keymap.rs"]
mod hid_keymap;

#[path = "hid_settings.rs"]
mod hid_settings;

#[cfg(not(target_arch = "wasm32"))]
pub struct HidDevice {
    device: hidapi::HidDevice,
}

#[cfg(not(target_arch = "wasm32"))]
impl HidDevice {
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
        let bytes_read = self
            .device
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
        let kb_id = u64::from_le_bytes([
            resp[4], resp[5], resp[6], resp[7], resp[8], resp[9], resp[10], resp[11],
        ]);
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

        // Decompress: vial uses Python lzma which defaults to XZ container format
        let mut decompressed = Vec::new();
        let xz_result = lzma_rs::xz_decompress(&mut &payload[..], &mut decompressed);
        if xz_result.is_err() {
            // fallback: try raw LZMA
            decompressed.clear();
            lzma_rs::lzma_decompress(&mut &payload[..], &mut decompressed)
                .context("Failed to decompress vial definition (tried xz and lzma)")?;
        }

        let json_str =
            std::str::from_utf8(&decompressed).context("Vial definition is not valid UTF-8")?;

        let value: serde_json::Value =
            serde_json::from_str(json_str).context("Failed to parse vial JSON")?;
        Ok(value)
    }

    /// Check if keyboard is unlocked
    /// Returns (unlocked, unlock_keys: Vec<(row,col)>)
    pub fn get_unlock_status(&self) -> Result<(bool, Vec<(u8, u8)>)> {
        let resp = self.usb_send(&[CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_UNLOCK_STATUS])?;
        // resp[0] = unlocked (1=yes), resp[1] = unlock_in_progress
        // resp[2..] = pairs of (row, col), rest filled with 0xFF
        let unlocked = resp[0] == 1;
        let mut keys = Vec::new();
        let mut i = 2;
        while i + 1 < resp.len() {
            let row = resp[i];
            let col = resp[i + 1];
            if row == 0xFF && col == 0xFF {
                break;
            }
            keys.push((row, col));
            i += 2;
        }
        Ok((unlocked, keys))
    }

    /// Start unlock sequence — returns keys to hold (row, col pairs)
    pub fn unlock_start(&self) -> Result<()> {
        self.usb_send(&[CMD_VIA_VIAL_PREFIX, CMD_VIAL_UNLOCK_START])?;
        Ok(())
    }

    /// Poll unlock status — returns (unlocked, in_progress)
    /// Returns (unlocked, in_progress, counter)
    pub fn unlock_poll(&self) -> Result<(bool, bool, u8)> {
        let resp = self.usb_send(&[CMD_VIA_VIAL_PREFIX, CMD_VIAL_UNLOCK_POLL])?;
        // resp[0] = unlocked, resp[1] = in_progress, resp[2] = counter
        Ok((resp[0] == 1, resp[1] == 1, resp[2]))
    }

    /// Lock the keyboard
    pub fn lock(&self) -> Result<()> {
        self.usb_send(&[CMD_VIA_VIAL_PREFIX, CMD_VIAL_LOCK])?;
        Ok(())
    }

}
