/// ZMK Studio protocol implementation over USB Serial (CDC/ACM).
///
/// Framing: SoF=0xAB, Esc=0xAC, EoF=0xAD
/// Payload: protobuf (prost)
/// Request/response matching by request_id.

use anyhow::{bail, Context, Result};
use prost::Message;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use crate::zmk_proto;

// Framing bytes
const SOF: u8 = 0xAB;
const ESC: u8 = 0xAC;
const EOF: u8 = 0xAD;

static NEXT_REQUEST_ID: AtomicU32 = AtomicU32::new(1);

fn next_request_id() -> u32 {
    NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

/// Behavior info fetched from the device, used for the dynamic picker.
#[derive(Debug, Clone)]
pub struct BehaviorInfo {
    pub id: u32,
    pub display_name: String,
    pub metadata: Vec<zmk_proto::behaviors::BehaviorBindingParametersSet>,
}

/// ZMK binding — matches the protobuf BehaviorBinding.
#[derive(Debug, Clone, PartialEq)]
pub struct ZmkBinding {
    pub behavior_id: i32,
    pub param1: u32,
    pub param2: u32,
}

impl ZmkBinding {
    pub fn none() -> Self {
        Self { behavior_id: -1, param1: 0, param2: 0 }
    }

    pub fn is_none(&self) -> bool {
        self.behavior_id == -1
    }

    pub fn from_proto(b: &zmk_proto::keymap::BehaviorBinding) -> Self {
        Self {
            behavior_id: b.behavior_id,
            param1: b.param1,
            param2: b.param2,
        }
    }

    pub fn to_proto(&self) -> zmk_proto::keymap::BehaviorBinding {
        zmk_proto::keymap::BehaviorBinding {
            behavior_id: self.behavior_id,
            param1: self.param1,
            param2: self.param2,
        }
    }
}

/// Encode payload with ZMK Studio framing.
fn frame_encode(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 4);
    out.push(SOF);
    for &b in payload {
        if b == SOF || b == ESC || b == EOF {
            out.push(ESC);
            out.push(b);
        } else {
            out.push(b);
        }
    }
    out.push(EOF);
    out
}

/// Decode a framed message, stripping SoF/EoF and unescaping.
fn frame_decode(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        bail!("Empty frame");
    }
    let mut out = Vec::with_capacity(data.len());
    let mut escaped = false;
    for &b in data {
        if escaped {
            out.push(b);
            escaped = false;
        } else if b == ESC {
            escaped = true;
        } else if b == SOF || b == EOF {
            // skip framing bytes
        } else {
            out.push(b);
        }
    }
    Ok(out)
}

pub struct ZmkConnection {
    port: Box<dyn serialport::SerialPort>,
    pub device_name: String,
    pub behaviors: Vec<BehaviorInfo>,
}

impl ZmkConnection {
    /// Open a serial port and establish ZMK Studio connection.
    pub fn open(port_name: &str) -> Result<Self> {
        let port = serialport::new(port_name, 115200)
            .timeout(Duration::from_secs(5))
            .open()
            .with_context(|| format!("Failed to open serial port {port_name}"))?;

        let mut conn = Self {
            port,
            device_name: String::new(),
            behaviors: Vec::new(),
        };

        // Get device info
        let info = conn.get_device_info()?;
        conn.device_name = info.name.clone();
        log::info!("ZMK device: {}", info.name);

        // Check lock state — must be unlocked
        let lock_state = conn.get_lock_state()?;
        if lock_state == zmk_proto::core::LockState::Locked as i32 {
            log::warn!("ZMK device is locked — some operations may fail");
        }

        Ok(conn)
    }

    /// Send a framed protobuf request and receive a framed response.
    fn send_request(&mut self, request: &zmk_proto::Request) -> Result<zmk_proto::Response> {
        let payload = request.encode_to_vec();
        let frame = frame_encode(&payload);

        self.port.write_all(&frame).context("Serial write failed")?;
        self.port.flush().context("Serial flush failed")?;

        // Read response frame
        let response_bytes = self.read_frame()?;
        let decoded = frame_decode(&response_bytes)?;
        let response = zmk_proto::Response::decode(&decoded[..])
            .context("Failed to decode ZMK response protobuf")?;

        Ok(response)
    }

    /// Read bytes until a complete frame (SOF...EOF) is received.
    fn read_frame(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(512);
        let mut single = [0u8; 1];
        let mut in_frame = false;
        let mut escaped = false;

        loop {
            match self.port.read(&mut single) {
                Ok(1) => {
                    let b = single[0];
                    if !in_frame {
                        if b == SOF {
                            in_frame = true;
                            buf.clear();
                            buf.push(b);
                        }
                        // else skip bytes before SOF
                    } else {
                        buf.push(b);
                        if escaped {
                            escaped = false;
                        } else if b == ESC {
                            escaped = true;
                        } else if b == EOF {
                            return Ok(buf);
                        }
                    }
                }
                Ok(0) => bail!("Serial port returned 0 bytes"),
                Ok(_) => unreachable!(),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    bail!("Serial read timeout waiting for ZMK response");
                }
                Err(e) => bail!("Serial read error: {e}"),
            }
        }
    }

    /// Send a typed request and get the matching RequestResponse.
    fn rpc(
        &mut self,
        subsystem: zmk_proto::request::Subsystem,
    ) -> Result<zmk_proto::RequestResponse> {
        let id = next_request_id();
        let request = zmk_proto::Request {
            request_id: id,
            subsystem: Some(subsystem),
        };

        // We may receive notifications before the actual response — skip them.
        for _attempt in 0..10 {
            let response = self.send_request(&request)?;
            match response.r#type {
                Some(zmk_proto::response::Type::RequestResponse(rr)) => {
                    if rr.request_id != id {
                        log::warn!("ZMK request_id mismatch: expected {id}, got {}", rr.request_id);
                    }
                    return Ok(rr);
                }
                Some(zmk_proto::response::Type::Notification(n)) => {
                    log::debug!("Received ZMK notification (skipping): {:?}", n);
                    // Read next frame for the actual response
                    let response_bytes = self.read_frame()?;
                    let decoded = frame_decode(&response_bytes)?;
                    let resp2 = zmk_proto::Response::decode(&decoded[..])
                        .context("Failed to decode ZMK response")?;
                    if let Some(zmk_proto::response::Type::RequestResponse(rr)) = resp2.r#type {
                        return Ok(rr);
                    }
                }
                None => bail!("Empty ZMK response"),
            }
        }
        bail!("Failed to get response after multiple attempts")
    }

    // ── Core RPCs ───────────────────────────────────────────────────────

    pub fn get_device_info(&mut self) -> Result<zmk_proto::core::GetDeviceInfoResponse> {
        let rr = self.rpc(zmk_proto::request::Subsystem::Core(
            zmk_proto::core::Request {
                request_type: Some(zmk_proto::core::request::RequestType::GetDeviceInfo(true)),
            },
        ))?;

        match rr.subsystem {
            Some(zmk_proto::request_response::Subsystem::Core(core_resp)) => {
                match core_resp.response_type {
                    Some(zmk_proto::core::response::ResponseType::GetDeviceInfo(info)) => Ok(info),
                    other => bail!("Unexpected core response: {other:?}"),
                }
            }
            other => bail!("Unexpected subsystem response: {other:?}"),
        }
    }

    pub fn get_lock_state(&mut self) -> Result<i32> {
        let rr = self.rpc(zmk_proto::request::Subsystem::Core(
            zmk_proto::core::Request {
                request_type: Some(zmk_proto::core::request::RequestType::GetLockState(true)),
            },
        ))?;

        match rr.subsystem {
            Some(zmk_proto::request_response::Subsystem::Core(core_resp)) => {
                match core_resp.response_type {
                    Some(zmk_proto::core::response::ResponseType::GetLockState(state)) => {
                        Ok(state)
                    }
                    other => bail!("Unexpected core response: {other:?}"),
                }
            }
            other => bail!("Unexpected subsystem response: {other:?}"),
        }
    }

    // ── Behavior RPCs ───────────────────────────────────────────────────

    pub fn list_all_behaviors(&mut self) -> Result<Vec<u32>> {
        let rr = self.rpc(zmk_proto::request::Subsystem::Behaviors(
            zmk_proto::behaviors::Request {
                request_type: Some(
                    zmk_proto::behaviors::request::RequestType::ListAllBehaviors(true),
                ),
            },
        ))?;

        match rr.subsystem {
            Some(zmk_proto::request_response::Subsystem::Behaviors(resp)) => {
                match resp.response_type {
                    Some(zmk_proto::behaviors::response::ResponseType::ListAllBehaviors(
                        list,
                    )) => Ok(list.behaviors),
                    other => bail!("Unexpected behaviors response: {other:?}"),
                }
            }
            other => bail!("Unexpected subsystem response: {other:?}"),
        }
    }

    pub fn get_behavior_details(
        &mut self,
        behavior_id: u32,
    ) -> Result<zmk_proto::behaviors::GetBehaviorDetailsResponse> {
        let rr = self.rpc(zmk_proto::request::Subsystem::Behaviors(
            zmk_proto::behaviors::Request {
                request_type: Some(
                    zmk_proto::behaviors::request::RequestType::GetBehaviorDetails(
                        zmk_proto::behaviors::GetBehaviorDetailsRequest { behavior_id },
                    ),
                ),
            },
        ))?;

        match rr.subsystem {
            Some(zmk_proto::request_response::Subsystem::Behaviors(resp)) => {
                match resp.response_type {
                    Some(
                        zmk_proto::behaviors::response::ResponseType::GetBehaviorDetails(
                            details,
                        ),
                    ) => Ok(details),
                    other => bail!("Unexpected behaviors response: {other:?}"),
                }
            }
            other => bail!("Unexpected subsystem response: {other:?}"),
        }
    }

    /// Fetch all behavior IDs and their details.
    pub fn fetch_all_behaviors(&mut self) -> Result<()> {
        let ids = self.list_all_behaviors()?;
        log::info!("ZMK behaviors: {} total", ids.len());

        self.behaviors.clear();
        for id in ids {
            match self.get_behavior_details(id) {
                Ok(details) => {
                    log::info!("  behavior {}: {}", details.id, details.display_name);
                    self.behaviors.push(BehaviorInfo {
                        id: details.id,
                        display_name: details.display_name,
                        metadata: details.metadata,
                    });
                }
                Err(e) => {
                    log::warn!("Failed to get behavior details for id={id}: {e}");
                }
            }
        }
        Ok(())
    }

    // ── Keymap RPCs ─────────────────────────────────────────────────────

    pub fn get_physical_layouts(&mut self) -> Result<zmk_proto::keymap::PhysicalLayouts> {
        let rr = self.rpc(zmk_proto::request::Subsystem::Keymap(
            zmk_proto::keymap::Request {
                request_type: Some(
                    zmk_proto::keymap::request::RequestType::GetPhysicalLayouts(true),
                ),
            },
        ))?;

        match rr.subsystem {
            Some(zmk_proto::request_response::Subsystem::Keymap(resp)) => {
                match resp.response_type {
                    Some(zmk_proto::keymap::response::ResponseType::GetPhysicalLayouts(
                        layouts,
                    )) => Ok(layouts),
                    other => bail!("Unexpected keymap response: {other:?}"),
                }
            }
            other => bail!("Unexpected subsystem response: {other:?}"),
        }
    }

        /// Poll lock state until unlocked. Returns Ok(()) when unlocked, Err if timeout.
    pub fn wait_for_unlock(&mut self, timeout_secs: u64) -> Result<()> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
        loop {
            match self.get_lock_state() {
                Ok(v) if v == zmk_proto::core::LockState::Unlocked as i32 => return Ok(()),
                Ok(_) => {}
                Err(e) => log::warn!("get_lock_state error: {e}"),
            }
            if std::time::Instant::now() >= deadline {
                bail!("Timeout waiting for keyboard unlock");
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    pub fn get_keymap(&mut self) -> Result<zmk_proto::keymap::Keymap> {
        let rr = self.rpc(zmk_proto::request::Subsystem::Keymap(
            zmk_proto::keymap::Request {
                request_type: Some(zmk_proto::keymap::request::RequestType::GetKeymap(true)),
            },
        ))?;

        match rr.subsystem {
            Some(zmk_proto::request_response::Subsystem::Keymap(resp)) => {
                match resp.response_type {
                    Some(zmk_proto::keymap::response::ResponseType::GetKeymap(keymap)) => {
                        Ok(keymap)
                    }
                    other => bail!("Unexpected keymap response: {other:?}"),
                }
            }
            // UnlockRequired comes back as a Meta response with SimpleError
            Some(zmk_proto::request_response::Subsystem::Meta(meta)) => {
                match meta.response_type {
                    Some(zmk_proto::meta::response::ResponseType::SimpleError(code))
                        if code == zmk_proto::meta::ErrorConditions::UnlockRequired as i32 =>
                    {
                        bail!("unlock_required")
                    }
                    other => bail!("Meta error in get_keymap: {other:?}"),
                }
            }
            other => bail!("Unexpected subsystem response: {other:?}"),
        }
    }

    pub fn set_layer_binding(
        &mut self,
        layer_id: u32,
        key_position: i32,
        binding: &ZmkBinding,
    ) -> Result<()> {
        let rr = self.rpc(zmk_proto::request::Subsystem::Keymap(
            zmk_proto::keymap::Request {
                request_type: Some(
                    zmk_proto::keymap::request::RequestType::SetLayerBinding(
                        zmk_proto::keymap::SetLayerBindingRequest {
                            layer_id,
                            key_position,
                            binding: Some(binding.to_proto()),
                        },
                    ),
                ),
            },
        ))?;

        match rr.subsystem {
            Some(zmk_proto::request_response::Subsystem::Keymap(resp)) => {
                match resp.response_type {
                    Some(zmk_proto::keymap::response::ResponseType::SetLayerBinding(code)) => {
                        if code == zmk_proto::keymap::SetLayerBindingResponseCode::Ok as i32 {
                            Ok(())
                        } else {
                            bail!("SetLayerBinding error code: {code}")
                        }
                    }
                    other => bail!("Unexpected keymap response: {other:?}"),
                }
            }
            other => bail!("Unexpected subsystem response: {other:?}"),
        }
    }

    pub fn add_layer(&mut self) -> Result<(u32, String)> {
        let rr = self.rpc(zmk_proto::request::Subsystem::Keymap(
            zmk_proto::keymap::Request {
                request_type: Some(zmk_proto::keymap::request::RequestType::AddLayer(
                    zmk_proto::keymap::AddLayerRequest {},
                )),
            },
        ))?;
        match rr.subsystem {
            Some(zmk_proto::request_response::Subsystem::Keymap(resp)) => {
                match resp.response_type {
                    Some(zmk_proto::keymap::response::ResponseType::AddLayer(add_resp)) => {
                        match add_resp.result {
                            Some(zmk_proto::keymap::add_layer_response::Result::Ok(details)) => {
                                let name = details.layer.as_ref().map(|l| l.name.clone()).unwrap_or_default();
                                Ok((details.index, name))
                            }
                            Some(zmk_proto::keymap::add_layer_response::Result::Err(code)) => {
                                bail!("AddLayer error: {code}")
                            }
                            None => bail!("Empty AddLayer response"),
                        }
                    }
                    other => bail!("Unexpected AddLayer response: {other:?}"),
                }
            }
            other => bail!("Unexpected subsystem: {other:?}"),
        }
    }

    pub fn remove_layer(&mut self, layer_index: u32) -> Result<()> {
        let rr = self.rpc(zmk_proto::request::Subsystem::Keymap(
            zmk_proto::keymap::Request {
                request_type: Some(zmk_proto::keymap::request::RequestType::RemoveLayer(
                    zmk_proto::keymap::RemoveLayerRequest { layer_index },
                )),
            },
        ))?;
        match rr.subsystem {
            Some(zmk_proto::request_response::Subsystem::Keymap(resp)) => {
                match resp.response_type {
                    Some(zmk_proto::keymap::response::ResponseType::RemoveLayer(rem_resp)) => {
                        match rem_resp.result {
                            Some(zmk_proto::keymap::remove_layer_response::Result::Ok(_)) => Ok(()),
                            Some(zmk_proto::keymap::remove_layer_response::Result::Err(code)) => {
                                bail!("RemoveLayer error: {code}")
                            }
                            None => bail!("Empty RemoveLayer response"),
                        }
                    }
                    other => bail!("Unexpected RemoveLayer response: {other:?}"),
                }
            }
            other => bail!("Unexpected subsystem: {other:?}"),
        }
    }

    pub fn set_layer_name(&mut self, layer_id: u32, name: &str) -> Result<()> {
        let rr = self.rpc(zmk_proto::request::Subsystem::Keymap(
            zmk_proto::keymap::Request {
                request_type: Some(zmk_proto::keymap::request::RequestType::SetLayerProps(
                    zmk_proto::keymap::SetLayerPropsRequest {
                        layer_id,
                        name: name.to_string(),
                    },
                )),
            },
        ))?;
        match rr.subsystem {
            Some(zmk_proto::request_response::Subsystem::Keymap(resp)) => {
                match resp.response_type {
                    Some(zmk_proto::keymap::response::ResponseType::SetLayerProps(code)) => {
                        if code == zmk_proto::keymap::SetLayerPropsResponseCode::Ok as i32 {
                            Ok(())
                        } else {
                            bail!("SetLayerProps error: {code}")
                        }
                    }
                    other => bail!("Unexpected response: {other:?}"),
                }
            }
            other => bail!("Unexpected subsystem: {other:?}"),
        }
    }

    pub fn save_changes(&mut self) -> Result<()> {
        let rr = self.rpc(zmk_proto::request::Subsystem::Keymap(
            zmk_proto::keymap::Request {
                request_type: Some(zmk_proto::keymap::request::RequestType::SaveChanges(true)),
            },
        ))?;

        match rr.subsystem {
            Some(zmk_proto::request_response::Subsystem::Keymap(resp)) => {
                match resp.response_type {
                    Some(zmk_proto::keymap::response::ResponseType::SaveChanges(save_resp)) => {
                        match save_resp.result {
                            Some(zmk_proto::keymap::save_changes_response::Result::Ok(_)) => {
                                Ok(())
                            }
                            Some(zmk_proto::keymap::save_changes_response::Result::Err(code)) => {
                                bail!("SaveChanges error code: {code}")
                            }
                            None => bail!("Empty save_changes response"),
                        }
                    }
                    other => bail!("Unexpected keymap response: {other:?}"),
                }
            }
            other => bail!("Unexpected subsystem response: {other:?}"),
        }
    }
}

pub(crate) fn zmk_behavior_kind(name: &str) -> &'static str {
    let normalized = name.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "none" | "no key" | "no-key" | "no operation" | "no op" => "none",
        "transparent" | "trans" | "trns" => "transparent",
        "key press" | "key" | "kp" => "key_press",
        "grave/escape" | "grave escape" | "grave-escape" | "gesc" => "grave_escape",
        "key repeat" | "repeat" => "key_repeat",
        "key toggle" => "key_toggle",
        "momentary layer" | "mo" => "momentary_layer",
        "toggle layer" | "tg" => "toggle_layer",
        "to layer" | "to" => "to_layer",
        "sticky layer" | "one-shot layer" | "osl" => "sticky_layer",
        "layer-tap" | "layer tap" | "lt" => "layer_tap",
        "mod-tap" | "mod tap" | "mt" => "mod_tap",
        "sticky key" | "one-shot key" | "one shot key" | "osm" => "sticky_key",
        "caps word" => "caps_word",
        "reset" => "reset",
        "bootloader" => "bootloader",
        "studio unlock" => "studio_unlock",
        "bluetooth" => "bluetooth",
        "output selection" | "output" => "output_selection",
        "external power" => "external_power",
        "soft off" | "soft_off" | "soft-off" | "z_so_off" => "soft_off",
        "rgb underglow" | "underglow" | "rgb_ug" | "rgb ug" | "rgb" => "rgb_underglow",
        "backlight" | "bcklight" | "bl" => "backlight",
        "mouse key press" | "mouse button" | "mkp" => "mouse_key_press",
        "mouse_move" | "mouse move" | "mouse-move" | "mmv" => "mouse_move",
        "mouse_scroll" | "mouse scroll" | "mouse-scroll" | "msc" => "mouse_scroll",
        _ => "unknown",
    }
}

fn zmk_mod_mask_to_vial_base(mask: u32) -> Option<u16> {
    match mask {
        0x01 => Some(0x0100),
        0x02 => Some(0x0200),
        0x04 => Some(0x0400),
        0x08 => Some(0x0800),
        0x10 => Some(0x1100),
        0x20 => Some(0x1200),
        0x40 => Some(0x1400),
        0x80 => Some(0x1800),
        0x03 => Some(0x0300),
        0x05 => Some(0x0500),
        0x06 => Some(0x0600),
        0x07 => Some(0x0700),
        0x0A => Some(0x0A00),
        0x0F => Some(0x0F00),
        _ => None,
    }
}

fn zmk_modifier_usage_to_vial_osm(usage: u32) -> Option<u16> {
    match usage {
        0x0007_00E0 | 0x01 => Some(0x52A1),
        0x0007_00E1 | 0x02 => Some(0x52A2),
        0x0007_00E2 | 0x04 => Some(0x52A4),
        0x0007_00E3 | 0x08 => Some(0x52A8),
        0x0007_00E4 | 0x10 => Some(0x52B1),
        0x0007_00E5 | 0x20 => Some(0x52B2),
        0x0007_00E6 | 0x40 => Some(0x52B4),
        0x0007_00E7 | 0x80 => Some(0x52B8),
        _ => None,
    }
}

fn zmk_modifier_usage_to_vial_mt_base(usage: u32) -> Option<u16> {
    match usage {
        0x0007_00E0 => Some(0x2100),
        0x0007_00E1 => Some(0x2200),
        0x0007_00E2 => Some(0x2400),
        0x0007_00E3 => Some(0x2800),
        0x0007_00E4 => Some(0x3100),
        0x0007_00E5 => Some(0x3200),
        0x0007_00E6 => Some(0x3400),
        0x0007_00E7 => Some(0x3800),
        _ => None,
    }
}

fn zmk_mouse_button_param_to_vial_value(param: u32) -> Option<u16> {
    match param {
        0x0009_0001 | 0x01 => Some(0x00D1),
        0x0009_0002 | 0x02 => Some(0x00D2),
        0x0009_0003 | 0x04 => Some(0x00D3),
        0x0009_0004 | 0x08 => Some(0x00D4),
        0x0009_0005 | 0x10 => Some(0x00D5),
        _ => None,
    }
}

fn zmk_space_cadet_equivalent(modifier: u32, tap: u32) -> Option<u16> {
    match (modifier, zmk_hid_usage_to_vial_value(tap)?) {
        (0x0007_00E0, 0x2226) => Some(0x7C18),
        (0x0007_00E4, 0x2227) => Some(0x7C19),
        (0x0007_00E1, 0x2226) => Some(0x7C1A),
        (0x0007_00E5, 0x2227) => Some(0x7C1B),
        (0x0007_00E2, 0x2226) => Some(0x7C1C),
        (0x0007_00E6, 0x2227) => Some(0x7C1D),
        (0x0007_00E5, 0x0028) => Some(0x7C1E),
        _ => None,
    }
}

fn zmk_hid_usage_to_vial_value(usage: u32) -> Option<u16> {
    let mod_mask = (usage >> 24) & 0xFF;
    let raw = usage & 0x00FF_FFFF;
    let page = raw >> 16;
    let usage_id = raw & 0xFFFF;

    let base = match page {
        0x07 => match usage_id {
            0x0004..=0x00A4 => Some(usage_id as u16),
            _ => None,
        },
        0x09 => match usage_id {
            0x01..=0x08 => Some(0x00D0 + usage_id as u16),
            _ => None,
        },
        0x01 => match usage_id {
            0x81 => Some(0x00A5),
            0x82 => Some(0x00A6),
            0x83 => Some(0x00A7),
            _ => None,
        },
        0x0C => match usage_id {
            0x006F => Some(0x00BD),
            0x0070 => Some(0x00BE),
            0x00B3 => Some(0x00BB),
            0x00B4 => Some(0x00BC),
            0x00B5 => Some(0x00AB),
            0x00B6 => Some(0x00AC),
            0x00B7 => Some(0x00AD),
            0x00B8 => Some(0x00B0),
            0x00CD => Some(0x00AE),
            0x00E2 => Some(0x00A8),
            0x00E9 => Some(0x00A9),
            0x00EA => Some(0x00AA),
            0x0183 => Some(0x00AF),
            0x018A => Some(0x00B1),
            0x0192 => Some(0x00B2),
            0x0194 => Some(0x00B3),
            0x0221 => Some(0x00B4),
            0x0223 => Some(0x00B5),
            0x0224 => Some(0x00B6),
            0x0225 => Some(0x00B7),
            0x0226 => Some(0x00B8),
            0x021A => Some(0x007A),
            0x021B => Some(0x007C),
            0x021C => Some(0x007B),
            0x021D => Some(0x007D),
            0x021F => Some(0x007E),
            0x0227 => Some(0x00B9),
            0x022A => Some(0x00BA),
            _ => None,
        },
        _ => None,
    }?;

    if mod_mask == 0 {
        Some(base)
    } else {
        zmk_mod_mask_to_vial_base(mod_mask).map(|mod_base| mod_base | base)
    }
}

fn zmk_hid_usage_label_with_vial_style(usage: u32, layer_names: &[String]) -> String {
    if let Some(value) = zmk_hid_usage_to_vial_value(usage) {
        crate::keycode::keycode_label_with_names(value, &[], layer_names)
    } else {
        hid_usage_label(usage)
    }
}

fn zmk_equivalent_vial_keycode(kind: &str, p1: u32, p2: u32) -> Option<u16> {
    match kind {
        "none" => Some(0x0000),
        "transparent" => Some(0x0001),
        "grave_escape" => Some(0x7C16),
        "bootloader" => Some(0x7C00),
        "caps_word" => Some(0x7C73),
        "key_repeat" => Some(0x7C79),
        "key_press" => zmk_hid_usage_to_vial_value(p1),
        "mouse_key_press" => zmk_mouse_button_param_to_vial_value(p1),
        "momentary_layer" if p1 < 32 => Some(0x5220 | p1 as u16),
        "toggle_layer" if p1 < 32 => Some(0x5260 | p1 as u16),
        "to_layer" if p1 < 32 => Some(0x5200 | p1 as u16),
        "sticky_layer" if p1 < 32 => Some(0x5280 | p1 as u16),
        "layer_tap" if p1 <= 0x0F => {
            zmk_hid_usage_to_vial_value(p2)
                .filter(|tap| *tap <= 0xFF)
                .map(|tap| 0x4000 | ((p1 as u16) << 8) | tap)
        }
        "mod_tap" => {
            if let Some(value) = zmk_space_cadet_equivalent(p1, p2) {
                Some(value)
            } else if let (Some(base), Some(tap)) = (
                zmk_modifier_usage_to_vial_mt_base(p1),
                zmk_hid_usage_to_vial_value(p2).filter(|tap| *tap <= 0xFF),
            ) {
                Some(base | tap)
            } else {
                None
            }
        }
        "sticky_key" => zmk_modifier_usage_to_vial_osm(p1),
        _ => None,
    }
}

fn zmk_axis_label(value: u32, kind: &str) -> Option<&'static str> {
    match (kind, value) {
        ("move", 0x0000_FDA8) => Some("Move\nUp"),
        ("move", 0x0000_0258) => Some("Move\nDown"),
        ("move", 0xFDA8_0000) => Some("Move\nLeft"),
        ("move", 0x0258_0000) => Some("Move\nRight"),
        ("scroll", 0x0000_000A) => Some("Scroll\nUp"),
        ("scroll", 0x0000_FFF6) => Some("Scroll\nDown"),
        ("scroll", 0xFFF6_0000) => Some("Scroll\nLeft"),
        ("scroll", 0x000A_0000) => Some("Scroll\nRight"),
        _ => None,
    }
}

fn zmk_axis_tooltip(value: u32, kind: &str) -> Option<&'static str> {
    match (kind, value) {
        ("move", 0x0000_FDA8) => Some("Mouse move up"),
        ("move", 0x0000_0258) => Some("Mouse move down"),
        ("move", 0xFDA8_0000) => Some("Mouse move left"),
        ("move", 0x0258_0000) => Some("Mouse move right"),
        ("scroll", 0x0000_000A) => Some("Mouse wheel scroll up"),
        ("scroll", 0x0000_FFF6) => Some("Mouse wheel scroll down"),
        ("scroll", 0xFFF6_0000) => Some("Mouse wheel scroll left"),
        ("scroll", 0x000A_0000) => Some("Mouse wheel scroll right"),
        _ => None,
    }
}

fn zmk_lighting_label(kind: &str, p1: u32) -> Option<&'static str> {
    match (kind, p1) {
        ("rgb_underglow", 0) => Some("RGB\nToggle"),
        ("rgb_underglow", 1) => Some("RGB\nOn"),
        ("rgb_underglow", 2) => Some("RGB\nOff"),
        ("rgb_underglow", 3) => Some("Hue+"),
        ("rgb_underglow", 4) => Some("Hue-"),
        ("rgb_underglow", 5) => Some("Sat+"),
        ("rgb_underglow", 6) => Some("Sat-"),
        ("rgb_underglow", 7) => Some("RGB\nBright+"),
        ("rgb_underglow", 8) => Some("RGB\nBright-"),
        ("rgb_underglow", 9) => Some("RGB\nSpeed+"),
        ("rgb_underglow", 10) => Some("RGB\nSpeed-"),
        ("rgb_underglow", 11) => Some("RGB\nEffect+"),
        ("rgb_underglow", 12) => Some("RGB\nEffect-"),
        ("backlight", 0) => Some("BL\nOn"),
        ("backlight", 1) => Some("BL\nOff"),
        ("backlight", 2) => Some("BL\nToggle"),
        ("backlight", 3) => Some("BL+"),
        ("backlight", 4) => Some("BL-"),
        ("backlight", 5) => Some("BL\nCycle"),
        _ => None,
    }
}

fn zmk_lighting_tooltip(kind: &str, p1: u32) -> Option<&'static str> {
    match (kind, p1) {
        ("rgb_underglow", 0) => Some("Toggle RGB underglow"),
        ("rgb_underglow", 1) => Some("Turn RGB underglow on"),
        ("rgb_underglow", 2) => Some("Turn RGB underglow off"),
        ("rgb_underglow", 3) => Some("Increase RGB hue"),
        ("rgb_underglow", 4) => Some("Decrease RGB hue"),
        ("rgb_underglow", 5) => Some("Increase RGB saturation"),
        ("rgb_underglow", 6) => Some("Decrease RGB saturation"),
        ("rgb_underglow", 7) => Some("Increase RGB brightness"),
        ("rgb_underglow", 8) => Some("Decrease RGB brightness"),
        ("rgb_underglow", 9) => Some("Increase RGB animation speed"),
        ("rgb_underglow", 10) => Some("Decrease RGB animation speed"),
        ("rgb_underglow", 11) => Some("Next RGB effect"),
        ("rgb_underglow", 12) => Some("Previous RGB effect"),
        ("backlight", 0) => Some("Turn backlight on"),
        ("backlight", 1) => Some("Turn backlight off"),
        ("backlight", 2) => Some("Toggle backlight"),
        ("backlight", 3) => Some("Increase backlight brightness"),
        ("backlight", 4) => Some("Decrease backlight brightness"),
        ("backlight", 5) => Some("Cycle backlight brightness"),
        _ => None,
    }
}

/// Get a display label for a ZMK binding given behavior info.
pub fn zmk_binding_label(binding: &ZmkBinding, behaviors: &[BehaviorInfo], layer_names: &[String]) -> String {
    if binding.is_none() {
        return "\u{2715}".to_string(); // ✕
    }

    let behavior = behaviors.iter().find(|b| b.id == binding.behavior_id as u32);
    let name = behavior.map(|b| b.display_name.as_str()).unwrap_or("???");
    let kind = zmk_behavior_kind(name);
    let p1 = binding.param1;
    let p2 = binding.param2;

    if let Some(value) = zmk_equivalent_vial_keycode(kind, p1, p2) {
        return crate::keycode::keycode_label_with_names(value, &[], layer_names);
    }

    let key = |u: u32| zmk_hid_usage_label_with_vial_style(u, layer_names);
    let layer = |n: u32| -> String {
        match layer_names.get(n as usize) {
            Some(s) if !s.is_empty() && s != &n.to_string() => format!("{}({})\n{}", "", n, s),
            _ => n.to_string(),
        }
    };
    let layer_label = |op: &str, n: u32| -> String {
        match layer_names.get(n as usize) {
            Some(s) if !s.is_empty() && s != &n.to_string() => format!("{}({})\n{}", op, n, s),
            _ => format!("{}({})", op, n),
        }
    };
    let _ = layer; // suppress unused

    match kind {
        // Simple key — just show the key label
        "key_press" => key(p1),
        // Transparent — like Vial TRNS
        "transparent" => crate::keycode::keycode_label_with_names(0x0001, &[], layer_names),
        // None / no key
        "none" => crate::keycode::keycode_label_with_names(0x0000, &[], layer_names),
        // Grave/Escape
        "grave_escape" => crate::keycode::keycode_label_with_names(0x7C16, &[], layer_names),
        // Key Repeat
        "key_repeat" => crate::keycode::keycode_label_with_names(0x7C79, &[], layer_names),
        // Key Toggle
        "key_toggle" => format!("KT\n{}", key(p1)),
        // Layer operations
        "momentary_layer"  => layer_label("MO", p1),
        "toggle_layer"     => layer_label("TG", p1),
        "to_layer"         => layer_label("TO", p1),
        "sticky_layer"     => layer_label("OSL", p1),
        // Layer-Tap: tap=key, hold=layer
        "layer_tap"        => {
            if p1 <= 0x0F {
                if let Some(tap) = zmk_hid_usage_to_vial_value(p2).filter(|tap| *tap <= 0xFF) {
                    crate::keycode::keycode_label_with_names(0x4000 | ((p1 as u16) << 8) | tap, &[], layer_names)
                } else {
                    format!("{}\n{}", layer_label("LT", p1), key(p2))
                }
            } else {
                format!("{}\n{}", layer_label("LT", p1), key(p2))
            }
        }
        // Mod-Tap: tap=key, hold=mod
        "mod_tap"          => {
            if let (Some(base), Some(tap)) = (
                zmk_modifier_usage_to_vial_mt_base(p1),
                zmk_hid_usage_to_vial_value(p2).filter(|tap| *tap <= 0xFF),
            ) {
                crate::keycode::keycode_label_with_names(base | tap, &[], layer_names)
            } else {
                format!("MT\n{}", key(p2))
            }
        }
        // Sticky Key (one-shot)
        "sticky_key"       => {
            if let Some(value) = zmk_modifier_usage_to_vial_osm(p1) {
                crate::keycode::keycode_label_with_names(value, &[], layer_names)
            } else {
                format!("SK\n{}", key(p1))
            }
        }
        // Caps Word
        "caps_word"        => crate::keycode::keycode_label_with_names(0x7C73, &[], layer_names),
        // Reset / Bootloader
        "reset"            => "Restart".to_string(),
        "bootloader"       => crate::keycode::keycode_label_with_names(0x7C00, &[], layer_names),
        // Studio Unlock
        "studio_unlock"    => "Unlock".to_string(),
        // Bluetooth — p1=action, p2=profile index for SEL
        "bluetooth" => match p1 {
            0 => "BT\nClear".to_string(),
            1 => "BT\nClear All".to_string(),
            2 => "BT\nNext".to_string(),
            3 => "BT\nPrev".to_string(),
            4 => format!("BT\n{}", p2),
            _ => format!("BT\n{}", p1),
        },
        // Output Selection — p1: 0=USB, 1=BLE
        "output_selection" => match p1 {
            0 => "USB".to_string(),
            1 => "BLE".to_string(),
            _ => "Output".to_string(),
        },
        // External Power / Power
        "external_power" => match p1 {
            0 => "Power\nOff".to_string(),
            _ => "Power\nOn".to_string(),
        },
        "soft_off" => "Soft\nOff".to_string(),
        // Lighting
        "rgb_underglow" | "backlight" => zmk_lighting_label(kind, p1)
            .unwrap_or(name)
            .to_string(),
        // Mouse
        "mouse_key_press"  => format!("Ms\n{}", key(p1)),
        "mouse_move"       => zmk_axis_label(p1, "move").unwrap_or("MsMove").to_string(),
        "mouse_scroll"     => zmk_axis_label(p1, "scroll").unwrap_or("MsScrl").to_string(),
        // Encoder
        name if name.contains("ENC") || name.contains("enc") || name.contains("Vol") => {
            name.chars().take(6).collect()
        }
        // Fallback: shorten display name
        _ => {
            if p1 == 0 && p2 == 0 {
                // No params — show shortened name
                name.chars().take(8).collect()
            } else if p2 == 0 {
                format!("{}\n{}", &name.chars().take(6).collect::<String>(), key(p1))
            } else {
                format!("{}\n{} {}", &name.chars().take(4).collect::<String>(), key(p1), key(p2))
            }
        }
    }
}

/// Get a human-readable tooltip for a ZMK binding.
pub fn zmk_binding_tooltip(binding: &ZmkBinding, behaviors: &[BehaviorInfo], layer_names: &[String]) -> String {
    if binding.is_none() {
        return crate::keycode::keycode_tooltip(0x0000, &[], layer_names);
    }

    let behavior = behaviors.iter().find(|b| b.id == binding.behavior_id as u32);
    let name = behavior.map(|b| b.display_name.as_str()).unwrap_or("Unknown");
    let kind = zmk_behavior_kind(name);
    let p1 = binding.param1;
    let p2 = binding.param2;

    if let Some(value) = zmk_equivalent_vial_keycode(kind, p1, p2) {
        return crate::keycode::keycode_tooltip(value, &[], layer_names);
    }

    let layer_display = |n: u32| -> String {
        match layer_names.get(n as usize) {
            Some(s) if !s.is_empty() && s != &n.to_string() => format!("\"{}\" ({})", s, n),
            _ => format!("layer {}", n),
        }
    };
    let key_name = |u: u32| hid_usage_label(u);

    match kind {
        "key_press"        => format!("Press {}", key_name(p1)),
        "transparent"      => "Transparent — uses the binding from the layer below".to_string(),
        "none"             => "No key — does nothing".to_string(),
        "grave_escape"     => format!("Grave/Escape — sends Esc normally, ` when Shift or {} held", crate::keycode::gui_mod_name()),
        "key_repeat"       => "Repeat — repeats the last pressed key".to_string(),
        "key_toggle"       => format!("Key Toggle — toggles {} on/off", key_name(p1)),
        "caps_word"        => "Caps Word — capitalises next word, then deactivates".to_string(),
        "sticky_key"       => format!("One-Shot {} — activates for the next keypress only", key_name(p1)),
        "momentary_layer"  => format!("MO({}) — activate {} while held, return on release", p1, layer_display(p1)),
        "toggle_layer"     => format!("TG({}) — toggle {} on/off", p1, layer_display(p1)),
        "to_layer"         => format!("TO({}) — switch to {} and stay", p1, layer_display(p1)),
        "sticky_layer"     => format!("OSL({}) — activate {} for next keypress only", p1, layer_display(p1)),
        "layer_tap"        => format!("Layer Tap — tap for {}, hold to activate {}", key_name(p2), layer_display(p1)),
        "mod_tap"          => format!("Mod Tap — tap for {}, hold for {}", key_name(p2), key_name(p1)),
        "bootloader"       => "Bootloader — put keyboard into flash mode".to_string(),
        "reset"            => "Restart the keyboard".to_string(),
        "studio_unlock"    => "Unlock editing — allow live keymap changes".to_string(),
        "external_power"   => match p1 { 0 => "External power off".to_string(), _ => "External power on".to_string() },
        "soft_off"         => "Turn the keyboard off until hardware wake/reset".to_string(),
        "rgb_underglow" | "backlight" => zmk_lighting_tooltip(kind, p1)
            .unwrap_or(name)
            .to_string(),
        "bluetooth" => match p1 {
            0 => "Bluetooth: forget current profile".to_string(),
            1 => "Bluetooth: forget all profiles".to_string(),
            2 => "Bluetooth: next profile".to_string(),
            3 => "Bluetooth: previous profile".to_string(),
            4 => format!("Bluetooth: profile {}", p2),
            _ => format!("Bluetooth action {}", p1),
        },
        "output_selection" => match p1 {
            0 => "Output: USB".to_string(),
            1 => "Output: Bluetooth".to_string(),
            _ => "Output selection".to_string(),
        },
        "mouse_key_press"  => zmk_mouse_button_param_to_vial_value(p1)
            .map(|value| crate::keycode::keycode_tooltip(value, &[], layer_names))
            .unwrap_or_else(|| format!("Mouse button: {}", key_name(p1))),
        "mouse_move"       => zmk_axis_tooltip(p1, "move").unwrap_or("Mouse cursor movement").to_string(),
        "mouse_scroll"     => zmk_axis_tooltip(p1, "scroll").unwrap_or("Mouse scroll wheel").to_string(),
        _ => {
            if p1 == 0 && p2 == 0 {
                name.to_string()
            } else if p2 == 0 {
                format!("{}: {}", name, key_name(p1))
            } else {
                format!("{}: {} + {}", name, key_name(p1), key_name(p2))
            }
        }
    }
}

fn param_label(
    value: u32,
    descriptions: &[zmk_proto::behaviors::BehaviorParameterValueDescription],
) -> String {
    use zmk_proto::behaviors::behavior_parameter_value_description::ValueType;

    // Check if value matches a named constant
    for desc in descriptions {
        match &desc.value_type {
            Some(ValueType::Constant(c)) if *c == value => {
                return desc.name.clone();
            }
            Some(ValueType::Nil(_)) if value == 0 => {
                return String::new();
            }
            Some(ValueType::HidUsage(_)) => {
                return hid_usage_label(value);
            }
            Some(ValueType::LayerId(_)) => {
                return format!("L{value}");
            }
            _ => {}
        }
    }

    if value == 0 {
        String::new()
    } else {
        format!("0x{value:04X}")
    }
}

fn param_description(
    value: u32,
    descriptions: &[zmk_proto::behaviors::BehaviorParameterValueDescription],
    layer_names: &[String],
) -> String {
    use zmk_proto::behaviors::behavior_parameter_value_description::ValueType;

    for desc in descriptions {
        match &desc.value_type {
            Some(ValueType::Constant(c)) if *c == value => {
                return format!("{} (constant)", desc.name);
            }
            Some(ValueType::Nil(_)) => {
                return "none".to_string();
            }
            Some(ValueType::HidUsage(_)) => {
                return format!("HID usage: {}", hid_usage_label(value));
            }
            Some(ValueType::LayerId(_)) => {
                let name = layer_names
                    .get(value as usize)
                    .filter(|n| !n.is_empty() && *n != &value.to_string())
                    .map(|n| format!("Layer {value} ({n})"))
                    .unwrap_or_else(|| format!("Layer {value}"));
                return name;
            }
            Some(ValueType::Range(r)) => {
                return format!("{value} (range {}..{})", r.min, r.max);
            }
            _ => {}
        }
    }

    format!("0x{value:04X}")
}

/// Convert HID usage code to a short label.
fn hid_usage_label(usage: u32) -> String {
    // ZMK encodes HID usage as (page << 16) | usage_id
    let page = usage >> 16;
    let usage_id = usage & 0xFFFF;

    // Mouse buttons (0x09).
    if page == 0x09 {
        return match usage_id {
            0x01 => "Left".to_string(),
            0x02 => "Right".to_string(),
            0x03 => "Middle".to_string(),
            0x04 => "Back".to_string(),
            0x05 => "Forward".to_string(),
            _ => format!("Mouse{}", usage_id),
        };
    }

    // Generic desktop system controls (0x01).
    if page == 0x01 {
        return match usage_id {
            0x81 => "Power".to_string(),
            0x82 => "Sleep".to_string(),
            0x83 => "Wake".to_string(),
            _ => format!("Sys:{:02X}", usage_id),
        };
    }

    // Consumer page (0x0C) — media/volume/app/browser keys.
    if page == 0x0C || (page == 0 && usage_id >= 0xB0 && usage_id <= 0xFF) {
        return match usage_id {
            0x6F => "Bri+".to_string(),
            0x70 => "Bri-".to_string(),
            0xB3 => "Fwd".to_string(),
            0xB4 => "Rew".to_string(),
            0xB5 => "Next".to_string(),
            0xB6 => "Prev".to_string(),
            0xB7 => "Stop".to_string(),
            0xB8 => "Eject".to_string(),
            0xCD => "Play".to_string(),
            0xE2 => "Mute".to_string(),
            0xE9 => "Vol+".to_string(),
            0xEA => "Vol-".to_string(),
            0x183 => "Media".to_string(),
            0x18A => "Mail".to_string(),
            0x192 => "Calc".to_string(),
            0x194 => "Files".to_string(),
            0x221 => "Search".to_string(),
            0x223 => "Home".to_string(),
            0x224 => "Back".to_string(),
            0x225 => "Forward".to_string(),
            0x226 => "Stop".to_string(),
            0x227 => "Refresh".to_string(),
            0x22A => "Favs".to_string(),
            _ => format!("C:{:02X}", usage_id),
        };
    }

    let usage = if usage > 0xFFFF { usage_id } else { usage };
    match usage {
        0x00 => String::new(),
        0x04..=0x1D => {
            // Letters A-Z
            let ch = (b'A' + (usage as u8 - 0x04)) as char;
            ch.to_string()
        }
        0x1E..=0x26 => {
            // Numbers 1-9
            let n = usage - 0x1E + 1;
            if n <= 9 { n.to_string() } else { "0".to_string() }
        }
        0x27 => "0".to_string(),
        0x28 => "\u{21B5}".to_string(), // ↵ Enter
        0x29 => "Esc".to_string(),
        0x2A => "\u{232B}".to_string(), // ⌫ Backspace
        0x2B => "Tab".to_string(),
        0x2C => "Spc".to_string(),
        0x2D => "_\n-".to_string(),
        0x2E => "+\n=".to_string(),
        0x2F => "{\n[".to_string(),
        0x30 => "}\n]".to_string(),
        0x31 => "|\n\\".to_string(),
        0x33 => ":\n;".to_string(),
        0x34 => "\"\n'".to_string(),
        0x35 => "~\n`".to_string(),
        0x36 => "<\n,".to_string(),
        0x37 => ">\n.".to_string(),
        0x38 => "?\n/".to_string(),
        0x39 => "Caps".to_string(),
        0x3A..=0x45 => format!("F{}", usage - 0x3A + 1),
        0x46 => "PrtSc".to_string(),
        0x47 => "ScrLk".to_string(),
        0x48 => "Pause".to_string(),
        0x49 => "Ins".to_string(),
        0x4A => "Home".to_string(),
        0x4B => "PgUp".to_string(),
        0x4C => "Del".to_string(),
        0x4D => "End".to_string(),
        0x4E => "PgDn".to_string(),
        0x4F => "\u{2192}".to_string(), // →
        0x50 => "\u{2190}".to_string(), // ←
        0x51 => "\u{2193}".to_string(), // ↓
        0x52 => "\u{2191}".to_string(), // ↑
        // Modifiers
        0xE0 => "Left\nCtrl".to_string(),
        0xE1 => "Left\nShift".to_string(),
        0xE2 => "Left\nAlt".to_string(),
        0xE3 => { #[cfg(target_os = "macos")] { "Left\nCmd".to_string() } #[cfg(target_os = "windows")] { "Left\nWin".to_string() } #[cfg(not(any(target_os = "macos", target_os = "windows")))] { "Left\nSuper".to_string() } },
        0xE4 => "Right\nCtrl".to_string(),
        0xE5 => "Right\nShift".to_string(),
        0xE6 => "Right\nAlt".to_string(),
        0xE7 => { #[cfg(target_os = "macos")] { "Right\nCmd".to_string() } #[cfg(target_os = "windows")] { "Right\nWin".to_string() } #[cfg(not(any(target_os = "macos", target_os = "windows")))] { "Right\nSuper".to_string() } },
        _ => format!("0x{usage:02X}"),
    }
}
