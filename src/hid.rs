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

    pub fn open_fresh_for(device: &crate::device::Device) -> Result<Self> {
        let api = hidapi::HidApi::new().context("Failed to init hidapi")?;
        let selected_name = device.name.as_str();
        let mut candidates = api.device_list().filter(|info| {
            info.usage_page() == 0xFF60
                && info.usage() == 0x61
                && info.vendor_id() == device.vendor_id
                && info.product_id() == device.product_id
        });

        if let Some(info) = candidates.find(|info| {
            info.product_string()
                .map(|name| name == selected_name)
                .unwrap_or(false)
        }) {
            return info
                .open_device(&api)
                .map(|device| Self { device })
                .context("Failed to open HID device");
        }

        for info in api.device_list() {
            if info.usage_page() == 0xFF60
                && info.usage() == 0x61
                && info.vendor_id() == device.vendor_id
                && info.product_id() == device.product_id
            {
                return info
                    .open_device(&api)
                    .map(|device| Self { device })
                    .context("Failed to open HID device");
            }
        }

        anyhow::bail!("HID device disappeared during reconnect")
    }

    /// Send exactly MSG_LEN bytes (with 0x00 report ID prepended), receive MSG_LEN bytes back.
    fn usb_send(&self, data: &[u8]) -> Result<[u8; MSG_LEN]> {
        if data.len() > MSG_LEN {
            bail!(
                "HID command too long — {} bytes, max {} bytes",
                data.len(),
                MSG_LEN
            );
        }

        let mut write_buf = [0u8; MSG_LEN + 1];
        write_buf[0] = 0x00; // report ID
        write_buf[1..1 + data.len()].copy_from_slice(data);

        let bytes_written = self.device.write(&write_buf).context("HID write failed")?;
        if bytes_written != write_buf.len() {
            bail!(
                "HID short write — wrote {} bytes, expected {} bytes",
                bytes_written,
                write_buf.len()
            );
        }

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
        if bytes_read != MSG_LEN && bytes_read != MSG_LEN + 1 {
            bail!(
                "HID invalid response length — read {} bytes, expected {} or {} bytes",
                bytes_read,
                MSG_LEN,
                MSG_LEN + 1
            );
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
