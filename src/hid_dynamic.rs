use super::hid_protocol::*;
use super::HidDevice;
use anyhow::Result;

#[cfg(not(target_arch = "wasm32"))]
impl HidDevice {
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

}
