/// Vial protocol implementation over HID.
/// Based on vial-gui Python source: protocol/keyboard_comm.py
use anyhow::{bail, Context, Result};

/// Raw HID packet size (without report ID byte)
const MSG_LEN: usize = 32;

// VIA commands
const CMD_VIA_GET_PROTOCOL_VERSION: u8 = 0x01;
const CMD_VIA_GET_KEYBOARD_VALUE: u8 = 0x02;
const CMD_VIA_SET_KEYBOARD_VALUE: u8 = 0x03;
const CMD_VIA_GET_KEYCODE: u8 = 0x04;
const CMD_VIA_SET_KEYCODE: u8 = 0x05;
const CMD_VIA_LIGHTING_SET_VALUE: u8 = 0x07;
const CMD_VIA_LIGHTING_GET_VALUE: u8 = 0x08;
const CMD_VIA_LIGHTING_SAVE: u8 = 0x09;
const CMD_VIA_GET_LAYER_COUNT: u8 = 0x11;
const CMD_VIA_KEYMAP_GET_BUFFER: u8 = 0x12;
const CMD_VIA_MACRO_GET_COUNT: u8 = 0x0C;
const CMD_VIA_MACRO_GET_BUFFER_SIZE: u8 = 0x0D;
const CMD_VIA_MACRO_GET_BUFFER: u8 = 0x0E;
const CMD_VIA_MACRO_SET_BUFFER: u8 = 0x0F;
const CMD_VIA_VIAL_PREFIX: u8 = 0xFE;

const VIA_LAYOUT_OPTIONS: u8 = 0x02;
const VIA_SWITCH_MATRIX_STATE: u8 = 0x03;
const QMK_BACKLIGHT_BRIGHTNESS: u8 = 0x09;
const QMK_BACKLIGHT_EFFECT: u8 = 0x0A;
const QMK_RGBLIGHT_BRIGHTNESS: u8 = 0x80;
const QMK_RGBLIGHT_EFFECT: u8 = 0x81;
const QMK_RGBLIGHT_EFFECT_SPEED: u8 = 0x82;
const QMK_RGBLIGHT_COLOR: u8 = 0x83;
const VIALRGB_GET_INFO: u8 = 0x40;
const VIALRGB_GET_MODE: u8 = 0x41;
const VIALRGB_GET_SUPPORTED: u8 = 0x42;
const VIALRGB_SET_MODE: u8 = 0x41;

// Vial sub-commands (used after CMD_VIA_VIAL_PREFIX)
const CMD_VIAL_GET_KEYBOARD_ID: u8 = 0x00;
const CMD_VIAL_GET_SIZE: u8 = 0x01;
const CMD_VIAL_GET_DEFINITION: u8 = 0x02;
const CMD_VIAL_GET_ENCODER: u8 = 0x03;
const CMD_VIAL_SET_ENCODER: u8 = 0x04;
const CMD_VIAL_GET_UNLOCK_STATUS: u8 = 0x05;
const CMD_VIAL_UNLOCK_START: u8 = 0x06;
const CMD_VIAL_UNLOCK_POLL: u8 = 0x07;
const CMD_VIAL_LOCK: u8 = 0x08;
const CMD_VIAL_QMK_SETTINGS_QUERY: u8 = 0x09;
const CMD_VIAL_QMK_SETTINGS_GET: u8 = 0x0A;
const CMD_VIAL_QMK_SETTINGS_SET: u8 = 0x0B;
const CMD_VIAL_DYNAMIC_ENTRY_OP: u8 = 0x0D;
const DYNAMIC_VIAL_GET_NUM_ENTRIES: u8 = 0x00;
const DYNAMIC_VIAL_TAP_DANCE_GET: u8 = 0x01;
const DYNAMIC_VIAL_TAP_DANCE_SET: u8 = 0x02;
const DYNAMIC_VIAL_COMBO_GET: u8 = 0x03;
const DYNAMIC_VIAL_COMBO_SET: u8 = 0x04;
const DYNAMIC_VIAL_KEY_OVERRIDE_GET: u8 = 0x05;
const DYNAMIC_VIAL_KEY_OVERRIDE_SET: u8 = 0x06;
const DYNAMIC_VIAL_ALT_REPEAT_KEY_GET: u8 = 0x07;
const DYNAMIC_VIAL_ALT_REPEAT_KEY_SET: u8 = 0x08;

const BUFFER_FETCH_CHUNK: usize = 28;

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

    pub fn get_layout_options(&self) -> Result<u32> {
        let resp = self.usb_send(&[CMD_VIA_GET_KEYBOARD_VALUE, VIA_LAYOUT_OPTIONS])?;
        if resp.len() < 6 {
            anyhow::bail!("layout options response too short");
        }
        Ok(u32::from_be_bytes([resp[2], resp[3], resp[4], resp[5]]))
    }

    pub fn set_layout_options(&self, options: u32) -> Result<()> {
        let bytes = options.to_be_bytes();
        let _ = self.usb_send(&[
            CMD_VIA_SET_KEYBOARD_VALUE,
            VIA_LAYOUT_OPTIONS,
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
        ])?;
        Ok(())
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

    /// Get Vial dynamic entry counts and optional feature bits.
    /// Returns (tap_dance, combo, key_override, alt_repeat, feature_bits).
    pub fn get_dynamic_entry_counts(&self) -> Result<(u8, u8, u8, u8, u8)> {
        let resp = self.usb_send(&[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_DYNAMIC_ENTRY_OP,
            DYNAMIC_VIAL_GET_NUM_ENTRIES,
        ])?;
        Ok((resp[0], resp[1], resp[2], resp[3], resp[31]))
    }

    /// Get number of combo entries available
    pub fn get_combo_count(&self) -> Result<u8> {
        let (_, combo, _, _, _) = self.get_dynamic_entry_counts()?;
        Ok(combo)
    }

    /// Get combo entry: ([trigger_keys; 4], output_keycode)
    pub fn get_combo(&self, idx: u8) -> Result<([u16; 4], u16)> {
        let resp = self.usb_send(&[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_DYNAMIC_ENTRY_OP,
            DYNAMIC_VIAL_COMBO_GET,
            idx,
        ])?;
        if resp[0] != 0 {
            anyhow::bail!("combo get error: {}", resp[0]);
        }
        let mut keys = [0u16; 4];
        for i in 0..4 {
            let off = 1 + i * 2;
            keys[i] = u16::from_le_bytes([resp[off], resp[off + 1]]);
        }
        let output = u16::from_le_bytes([resp[9], resp[10]]);
        Ok((keys, output))
    }

    /// Set combo entry
    pub fn set_combo(&self, idx: u8, keys: [u16; 4], output: u16) -> Result<()> {
        let mut cmd = [0u8; 32];
        cmd[0] = CMD_VIA_VIAL_PREFIX;
        cmd[1] = CMD_VIAL_DYNAMIC_ENTRY_OP;
        cmd[2] = DYNAMIC_VIAL_COMBO_SET;
        cmd[3] = idx;
        for i in 0..4 {
            let [lo, hi] = keys[i].to_le_bytes();
            let off = 4 + i * 2;
            cmd[off] = lo;
            cmd[off + 1] = hi;
        }
        let [out_lo, out_hi] = output.to_le_bytes();
        cmd[12] = out_lo;
        cmd[13] = out_hi;
        let resp = self.usb_send(&cmd)?;
        if resp[0] != 0 {
            anyhow::bail!("combo set error: {}", resp[0]);
        }
        Ok(())
    }

    /// Get number of key override entries available
    pub fn get_key_override_count(&self) -> Result<u8> {
        let (_, _, key_override, _, _) = self.get_dynamic_entry_counts()?;
        Ok(key_override)
    }

    /// Get number of alt repeat key entries available
    pub fn get_alt_repeat_key_count(&self) -> Result<u8> {
        let (_, _, _, alt_repeat, _) = self.get_dynamic_entry_counts()?;
        Ok(alt_repeat)
    }

    /// Get key override entry:
    /// (trigger, replacement, layers, trigger_mods, negative_mod_mask, suppressed_mods, options)
    pub fn get_key_override(&self, idx: u8) -> Result<(u16, u16, u16, u8, u8, u8, u8)> {
        let resp = self.usb_send(&[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_DYNAMIC_ENTRY_OP,
            DYNAMIC_VIAL_KEY_OVERRIDE_GET,
            idx,
        ])?;
        if resp[0] != 0 {
            anyhow::bail!("key override get error: {}", resp[0]);
        }
        let trigger = u16::from_le_bytes([resp[1], resp[2]]);
        let replacement = u16::from_le_bytes([resp[3], resp[4]]);
        let layers = u16::from_le_bytes([resp[5], resp[6]]);
        Ok((
            trigger,
            replacement,
            layers,
            resp[7],
            resp[8],
            resp[9],
            resp[10],
        ))
    }

    /// Set key override entry
    pub fn set_key_override(
        &self,
        idx: u8,
        trigger: u16,
        replacement: u16,
        layers: u16,
        trigger_mods: u8,
        negative_mod_mask: u8,
        suppressed_mods: u8,
        options: u8,
    ) -> Result<()> {
        let mut cmd = [0u8; 32];
        cmd[0] = CMD_VIA_VIAL_PREFIX;
        cmd[1] = CMD_VIAL_DYNAMIC_ENTRY_OP;
        cmd[2] = DYNAMIC_VIAL_KEY_OVERRIDE_SET;
        cmd[3] = idx;
        cmd[4..6].copy_from_slice(&trigger.to_le_bytes());
        cmd[6..8].copy_from_slice(&replacement.to_le_bytes());
        cmd[8..10].copy_from_slice(&layers.to_le_bytes());
        cmd[10] = trigger_mods;
        cmd[11] = negative_mod_mask;
        cmd[12] = suppressed_mods;
        cmd[13] = options;
        let resp = self.usb_send(&cmd)?;
        if resp[0] != 0 {
            anyhow::bail!("key override set error: {}", resp[0]);
        }
        Ok(())
    }

    /// Get alt repeat key entry: (last_key, alt_key, allowed_mods, options)
    pub fn get_alt_repeat_key(&self, idx: u8) -> Result<(u16, u16, u8, u8)> {
        let resp = self.usb_send(&[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_DYNAMIC_ENTRY_OP,
            DYNAMIC_VIAL_ALT_REPEAT_KEY_GET,
            idx,
        ])?;
        if resp[0] != 0 {
            anyhow::bail!("alt repeat key get error: {}", resp[0]);
        }
        Ok((
            u16::from_le_bytes([resp[1], resp[2]]),
            u16::from_le_bytes([resp[3], resp[4]]),
            resp[5],
            resp[6],
        ))
    }

    /// Set alt repeat key entry
    pub fn set_alt_repeat_key(
        &self,
        idx: u8,
        keycode: u16,
        alt_keycode: u16,
        allowed_mods: u8,
        options: u8,
    ) -> Result<()> {
        let mut cmd = [0u8; 32];
        cmd[0] = CMD_VIA_VIAL_PREFIX;
        cmd[1] = CMD_VIAL_DYNAMIC_ENTRY_OP;
        cmd[2] = DYNAMIC_VIAL_ALT_REPEAT_KEY_SET;
        cmd[3] = idx;
        cmd[4..6].copy_from_slice(&keycode.to_le_bytes());
        cmd[6..8].copy_from_slice(&alt_keycode.to_le_bytes());
        cmd[8] = allowed_mods;
        cmd[9] = options;
        let resp = self.usb_send(&cmd)?;
        if resp[0] != 0 {
            anyhow::bail!("alt repeat key set error: {}", resp[0]);
        }
        Ok(())
    }

    pub fn get_encoder(&self, layer: u8, idx: u8) -> Result<(u16, u16)> {
        let resp = self.usb_send(&[CMD_VIA_VIAL_PREFIX, CMD_VIAL_GET_ENCODER, layer, idx])?;
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
        let _ = self.usb_send(&[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_SET_ENCODER,
            layer,
            idx,
            direction,
            bytes[0],
            bytes[1],
        ])?;
        Ok(())
    }

    pub fn query_qmk_settings(&self) -> Result<Vec<u16>> {
        let mut supported = Vec::new();
        let mut cur = 0u16;

        loop {
            let mut cmd = [0u8; 32];
            cmd[0] = CMD_VIA_VIAL_PREFIX;
            cmd[1] = CMD_VIAL_QMK_SETTINGS_QUERY;
            cmd[2..4].copy_from_slice(&cur.to_le_bytes());
            let resp = self.usb_send(&cmd)?;

            let mut next = cur;
            for chunk in resp.chunks_exact(2) {
                let qsid = u16::from_le_bytes([chunk[0], chunk[1]]);
                next = next.max(qsid);
                if qsid != 0xFFFF {
                    supported.push(qsid);
                }
            }

            if next == 0xFFFF {
                break;
            }
            if next == cur {
                anyhow::bail!("qmk settings query did not advance from qsid: {cur}");
            }
            cur = next;
        }

        supported.sort_unstable();
        supported.dedup();
        Ok(supported)
    }

    pub fn get_qmk_setting_u8(&self, qsid: u16) -> Result<u8> {
        let mut cmd = [0u8; 32];
        cmd[0] = CMD_VIA_VIAL_PREFIX;
        cmd[1] = CMD_VIAL_QMK_SETTINGS_GET;
        cmd[2..4].copy_from_slice(&qsid.to_le_bytes());
        let resp = self.usb_send(&cmd)?;
        if resp[0] != 0 {
            anyhow::bail!("qmk setting get error or unsupported qsid: {qsid}");
        }
        Ok(resp[1])
    }

    pub fn set_qmk_setting_u8(&self, qsid: u16, value: u8) -> Result<()> {
        let mut cmd = [0u8; 32];
        cmd[0] = CMD_VIA_VIAL_PREFIX;
        cmd[1] = CMD_VIAL_QMK_SETTINGS_SET;
        cmd[2..4].copy_from_slice(&qsid.to_le_bytes());
        cmd[4] = value;
        let resp = self.usb_send(&cmd)?;
        if resp[0] != 0 {
            anyhow::bail!("qmk setting set error or unsupported qsid: {qsid}");
        }
        Ok(())
    }

    pub fn get_qmk_setting_u16(&self, qsid: u16) -> Result<u16> {
        let mut cmd = [0u8; 32];
        cmd[0] = CMD_VIA_VIAL_PREFIX;
        cmd[1] = CMD_VIAL_QMK_SETTINGS_GET;
        cmd[2..4].copy_from_slice(&qsid.to_le_bytes());
        let resp = self.usb_send(&cmd)?;
        if resp[0] != 0 {
            anyhow::bail!("qmk setting get error or unsupported qsid: {qsid}");
        }
        Ok(u16::from_le_bytes([resp[1], resp[2]]))
    }

    pub fn set_qmk_setting_u16(&self, qsid: u16, value: u16) -> Result<()> {
        let mut cmd = [0u8; 32];
        cmd[0] = CMD_VIA_VIAL_PREFIX;
        cmd[1] = CMD_VIAL_QMK_SETTINGS_SET;
        cmd[2..4].copy_from_slice(&qsid.to_le_bytes());
        cmd[4..6].copy_from_slice(&value.to_le_bytes());
        let resp = self.usb_send(&cmd)?;
        if resp[0] != 0 {
            anyhow::bail!("qmk setting set error or unsupported qsid: {qsid}");
        }
        Ok(())
    }

    pub fn get_qmk_setting_string(&self, qsid: u16) -> Result<String> {
        let mut cmd = [0u8; 32];
        cmd[0] = CMD_VIA_VIAL_PREFIX;
        cmd[1] = CMD_VIAL_QMK_SETTINGS_GET;
        cmd[2..4].copy_from_slice(&qsid.to_le_bytes());
        let resp = self.usb_send(&cmd)?;
        if resp[0] != 0 {
            anyhow::bail!("qmk setting get error or unsupported qsid: {qsid}");
        }
        let bytes = &resp[1..];
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        Ok(String::from_utf8_lossy(&bytes[..end]).trim().to_string())
    }

    pub fn set_qmk_setting_string(&self, qsid: u16, value: &str) -> Result<()> {
        let mut cmd = [0u8; 32];
        cmd[0] = CMD_VIA_VIAL_PREFIX;
        cmd[1] = CMD_VIAL_QMK_SETTINGS_SET;
        cmd[2..4].copy_from_slice(&qsid.to_le_bytes());

        let safe_value = value.replace('%', "%%");
        let bytes = safe_value.as_bytes();
        let max_len = cmd.len().saturating_sub(4);
        let copy_len = bytes.len().min(max_len.saturating_sub(1));
        cmd[4..4 + copy_len].copy_from_slice(&bytes[..copy_len]);
        cmd[4 + copy_len] = 0;

        let resp = self.usb_send(&cmd)?;
        if resp[0] != 0 {
            anyhow::bail!("qmk setting set error or unsupported qsid: {qsid}");
        }
        Ok(())
    }

    pub fn get_qmk_rgblight_brightness(&self) -> Result<u8> {
        let resp = self.usb_send(&[CMD_VIA_LIGHTING_GET_VALUE, QMK_RGBLIGHT_BRIGHTNESS])?;
        Ok(resp[2])
    }

    pub fn set_qmk_rgblight_brightness(&self, value: u8) -> Result<()> {
        self.usb_send(&[CMD_VIA_LIGHTING_SET_VALUE, QMK_RGBLIGHT_BRIGHTNESS, value])?;
        Ok(())
    }

    pub fn get_qmk_rgblight_effect(&self) -> Result<u8> {
        let resp = self.usb_send(&[CMD_VIA_LIGHTING_GET_VALUE, QMK_RGBLIGHT_EFFECT])?;
        Ok(resp[2])
    }

    pub fn set_qmk_rgblight_effect(&self, value: u8) -> Result<()> {
        self.usb_send(&[CMD_VIA_LIGHTING_SET_VALUE, QMK_RGBLIGHT_EFFECT, value])?;
        Ok(())
    }

    pub fn get_qmk_rgblight_effect_speed(&self) -> Result<u8> {
        let resp = self.usb_send(&[CMD_VIA_LIGHTING_GET_VALUE, QMK_RGBLIGHT_EFFECT_SPEED])?;
        Ok(resp[2])
    }

    pub fn set_qmk_rgblight_effect_speed(&self, value: u8) -> Result<()> {
        self.usb_send(&[CMD_VIA_LIGHTING_SET_VALUE, QMK_RGBLIGHT_EFFECT_SPEED, value])?;
        Ok(())
    }

    pub fn get_qmk_rgblight_color(&self) -> Result<(u8, u8)> {
        let resp = self.usb_send(&[CMD_VIA_LIGHTING_GET_VALUE, QMK_RGBLIGHT_COLOR])?;
        Ok((resp[2], resp[3]))
    }

    pub fn set_qmk_rgblight_color(&self, hue: u8, saturation: u8) -> Result<()> {
        self.usb_send(&[
            CMD_VIA_LIGHTING_SET_VALUE,
            QMK_RGBLIGHT_COLOR,
            hue,
            saturation,
        ])?;
        Ok(())
    }

    pub fn save_rgb(&self) -> Result<()> {
        self.usb_send(&[CMD_VIA_LIGHTING_SAVE])?;
        Ok(())
    }

    pub fn get_vialrgb_info(&self) -> Result<(u16, u8)> {
        let resp = self.usb_send(&[CMD_VIA_LIGHTING_GET_VALUE, VIALRGB_GET_INFO])?;
        let data = &resp[2..];
        Ok((u16::from_le_bytes([data[0], data[1]]), data[2]))
    }

    pub fn get_vialrgb_supported_effects(&self) -> Result<Vec<u16>> {
        let mut effects = vec![0u16];
        let mut max_effect = 0u16;
        while max_effect < 0xFFFF {
            let mut cmd = [0u8; MSG_LEN];
            cmd[0] = CMD_VIA_LIGHTING_GET_VALUE;
            cmd[1] = VIALRGB_GET_SUPPORTED;
            cmd[2..4].copy_from_slice(&max_effect.to_le_bytes());
            let resp = self.usb_send(&cmd)?;
            let mut batch_max = max_effect;
            for chunk in resp[2..].chunks_exact(2) {
                let value = u16::from_le_bytes([chunk[0], chunk[1]]);
                if value != 0xFFFF && !effects.contains(&value) {
                    effects.push(value);
                }
                batch_max = batch_max.max(value);
            }
            if batch_max == 0xFFFF || batch_max == max_effect {
                break;
            }
            max_effect = batch_max;
        }
        effects.sort_unstable();
        Ok(effects)
    }

    pub fn get_vialrgb_mode(&self) -> Result<(u16, u8, u8, u8, u8)> {
        let resp = self.usb_send(&[CMD_VIA_LIGHTING_GET_VALUE, VIALRGB_GET_MODE])?;
        let data = &resp[2..];
        Ok((
            u16::from_le_bytes([data[0], data[1]]),
            data[2],
            data[3],
            data[4],
            data[5],
        ))
    }

    pub fn set_vialrgb_mode(
        &self,
        mode: u16,
        speed: u8,
        hue: u8,
        saturation: u8,
        brightness: u8,
    ) -> Result<()> {
        let mut cmd = [0u8; MSG_LEN];
        cmd[0] = CMD_VIA_LIGHTING_SET_VALUE;
        cmd[1] = VIALRGB_SET_MODE;
        cmd[2..4].copy_from_slice(&mode.to_le_bytes());
        cmd[4] = speed;
        cmd[5] = hue;
        cmd[6] = saturation;
        cmd[7] = brightness;
        self.usb_send(&cmd)?;
        Ok(())
    }

    /// Get number of tap dance entries available
    pub fn get_tap_dance_count(&self) -> Result<u8> {
        let (tap_dance, _, _, _, _) = self.get_dynamic_entry_counts()?;
        Ok(tap_dance)
    }

    /// Get a tap dance entry: (on_tap, on_hold, on_double_tap, on_tap_hold, tapping_term)
    pub fn get_tap_dance(&self, idx: u8) -> Result<(u16, u16, u16, u16, u16)> {
        let resp = self.usb_send(&[
            CMD_VIA_VIAL_PREFIX,
            CMD_VIAL_DYNAMIC_ENTRY_OP,
            DYNAMIC_VIAL_TAP_DANCE_GET,
            idx,
        ])?;
        // resp[0] = status (0=ok), resp[1..] = entry data
        if resp[0] != 0 {
            anyhow::bail!("tap dance get error: {}", resp[0]);
        }
        let on_tap = u16::from_le_bytes([resp[1], resp[2]]);
        let on_hold = u16::from_le_bytes([resp[3], resp[4]]);
        let on_double_tap = u16::from_le_bytes([resp[5], resp[6]]);
        let on_tap_hold = u16::from_le_bytes([resp[7], resp[8]]);
        let tapping_term = u16::from_le_bytes([resp[9], resp[10]]);
        Ok((on_tap, on_hold, on_double_tap, on_tap_hold, tapping_term))
    }

    /// Set a tap dance entry
    pub fn set_tap_dance(
        &self,
        idx: u8,
        on_tap: u16,
        on_hold: u16,
        on_double_tap: u16,
        on_tap_hold: u16,
        tapping_term: u16,
    ) -> Result<()> {
        let mut cmd = [0u8; 32];
        cmd[0] = CMD_VIA_VIAL_PREFIX;
        cmd[1] = CMD_VIAL_DYNAMIC_ENTRY_OP;
        cmd[2] = DYNAMIC_VIAL_TAP_DANCE_SET;
        cmd[3] = idx;
        cmd[4..6].copy_from_slice(&on_tap.to_le_bytes());
        cmd[6..8].copy_from_slice(&on_hold.to_le_bytes());
        cmd[8..10].copy_from_slice(&on_double_tap.to_le_bytes());
        cmd[10..12].copy_from_slice(&on_tap_hold.to_le_bytes());
        cmd[12..14].copy_from_slice(&tapping_term.to_le_bytes());
        let resp = self.usb_send(&cmd)?;
        if resp[0] != 0 {
            anyhow::bail!("tap dance set error: {}", resp[0]);
        }
        Ok(())
    }

    pub fn get_switch_matrix(&self, rows: usize, cols: usize) -> Result<Vec<bool>> {
        let resp = self.usb_send(&[CMD_VIA_GET_KEYBOARD_VALUE, VIA_SWITCH_MATRIX_STATE])?;
        // Matrix data is packed row-by-row, with each row padded to whole bytes.
        // QMK matrix bits are little-endian inside each row byte: bit 0 = col 0.
        let data = &resp[2..];
        let total = rows * cols;
        let bytes_per_row = cols.div_ceil(8);
        let mut pressed = vec![false; total];

        for row in 0..rows {
            for col in 0..cols {
                let byte_idx = row * bytes_per_row + col / 8;
                let bit_idx = col % 8;
                if byte_idx < data.len() {
                    pressed[row * cols + col] = ((data[byte_idx] >> bit_idx) & 1) != 0;
                }
            }
        }

        Ok(pressed)
    }
}
