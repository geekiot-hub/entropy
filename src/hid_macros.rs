use super::hid_protocol::*;
use super::HidDevice;
use anyhow::Result;

#[cfg(not(target_arch = "wasm32"))]
impl HidDevice {
    /// Returns a bitmask of pressed keys: bits per row/col.
    /// Response: [cmd, value_id, data...] where data is ceil(rows*cols/8) bytes.
    /// Get macro count from device.
    pub fn get_macro_count(&self) -> Result<u8> {
        let resp = self.usb_send(&[CMD_VIA_MACRO_GET_COUNT])?;
        Ok(resp[1])
    }

    /// Get macro buffer size.
    pub fn get_macro_buffer_size(&self) -> Result<u16> {
        let resp = self.usb_send(&[CMD_VIA_MACRO_GET_BUFFER_SIZE])?;
        Ok(u16::from_be_bytes([resp[1], resp[2]]))
    }

    /// Read macro buffer (all macros as null-separated strings).
    pub fn get_macro_buffer(&self, size: u16) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(size as usize);
        let mut offset: u16 = 0;
        while offset < size {
            let chunk = 28u8; // max payload per HID message
            let mut cmd = [0u8; 32];
            cmd[0] = CMD_VIA_MACRO_GET_BUFFER;
            cmd[1] = (offset >> 8) as u8;
            cmd[2] = (offset & 0xFF) as u8;
            cmd[3] = chunk;
            let resp = self.usb_send(&cmd)?;
            let n = chunk.min((size - offset) as u8) as usize;
            buf.extend_from_slice(&resp[4..4 + n]);
            offset += chunk as u16;
        }
        Ok(buf)
    }

    /// Write macro buffer to device.
    pub fn set_macro_buffer(&self, data: &[u8]) -> Result<()> {
        let mut offset: u16 = 0;
        let total = data.len() as u16;
        while offset < total {
            let chunk = 28u16.min(total - offset);
            let mut cmd = [0u8; 32];
            cmd[0] = CMD_VIA_MACRO_SET_BUFFER;
            cmd[1] = (offset >> 8) as u8;
            cmd[2] = (offset & 0xFF) as u8;
            cmd[3] = chunk as u8;
            let start = offset as usize;
            cmd[4..4 + chunk as usize].copy_from_slice(&data[start..start + chunk as usize]);
            self.usb_send(&cmd)?;
            offset += chunk;
        }
        Ok(())
    }

    /// Parse macro buffer into individual macro strings.
    pub fn parse_macros(buf: &[u8], count: u8) -> Vec<String> {
        let mut macros = Vec::new();
        let mut start = 0;
        for _ in 0..count {
            let end = buf[start..]
                .iter()
                .position(|&b| b == 0)
                .map(|p| start + p)
                .unwrap_or(buf.len());
            let s = String::from_utf8_lossy(&buf[start..end]).to_string();
            macros.push(s);
            start = end + 1;
            if start >= buf.len() {
                break;
            }
        }
        while macros.len() < count as usize {
            macros.push(String::new());
        }
        macros
    }

    /// Encode macro strings into buffer (null-separated).
    pub fn encode_macros(macros: &[String], buf_size: u16) -> Vec<u8> {
        let mut buf = Vec::with_capacity(buf_size as usize);
        for (i, m) in macros.iter().enumerate() {
            buf.extend_from_slice(m.as_bytes());
            if i < macros.len() - 1 || buf.len() < buf_size as usize {
                buf.push(0);
            }
        }
        buf.resize(buf_size as usize, 0);
        buf
    }

}
