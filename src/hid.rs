/// Vial protocol implementation over HID.
/// Based on vial-gui Python source: protocol/keyboard_comm.py
use anyhow::{bail, Context, Result};

#[path = "hid_protocol.rs"]
mod hid_protocol;
use hid_protocol::MSG_LEN;

#[path = "hid_parse.rs"]
mod hid_parse;

#[path = "hid_dynamic.rs"]
mod hid_dynamic;

#[path = "hid_macros.rs"]
mod hid_macros;

#[path = "hid_keymap.rs"]
mod hid_keymap;

#[path = "hid_settings.rs"]
mod hid_settings;

#[path = "hid_vial.rs"]
mod hid_vial;

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

}
