/// Vial protocol implementation over HID.
/// Based on vial-gui Python source: protocol/keyboard_comm.py
use anyhow::{bail, Context, Result};
use std::time::Duration;

#[cfg(target_os = "windows")]
use std::io::{BufRead, BufReader, Write};

#[cfg(target_os = "windows")]
use std::process::{Child, ChildStdin, Command, Stdio};

#[cfg(target_os = "windows")]
use std::sync::{mpsc, Mutex};

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
    backend: HidBackend,
}

#[cfg(not(target_arch = "wasm32"))]
enum HidBackend {
    Local(hidapi::HidDevice),
    #[cfg(target_os = "windows")]
    Proxy(HidProxy),
}

#[cfg(target_os = "windows")]
struct HidProxy {
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    rx: Mutex<mpsc::Receiver<String>>,
}

#[cfg(target_os = "windows")]
#[derive(serde::Serialize, serde::Deserialize)]
struct ProxyResponse {
    ok: bool,
    data: Option<String>,
    error: Option<String>,
}

#[cfg(target_os = "windows")]
impl Drop for HidProxy {
    fn drop(&mut self) {
        if let Ok(child) = self.child.get_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn device_info_matches(
    info: &hidapi::DeviceInfo,
    device: &crate::device::Device,
    strict_identity: bool,
) -> bool {
    if info.usage_page() != 0xFF60
        || info.usage() != 0x61
        || info.vendor_id() != device.vendor_id
        || info.product_id() != device.product_id
    {
        return false;
    }

    if !strict_identity {
        return true;
    }

    let serial_matches = !device.serial_number.is_empty()
        && info
            .serial_number()
            .map(|serial| serial == device.serial_number)
            .unwrap_or(false);
    let product_matches = info
        .product_string()
        .map(|product| product == device.name)
        .unwrap_or(false);
    let manufacturer_matches = device.manufacturer.is_empty()
        || info
            .manufacturer_string()
            .map(|manufacturer| manufacturer == device.manufacturer)
            .unwrap_or(false);

    serial_matches || (product_matches && manufacturer_matches)
}

#[cfg(not(target_arch = "wasm32"))]
impl HidDevice {
    pub fn open(path: &str) -> Result<Self> {
        let api = hidapi::HidApi::new().context("Failed to init hidapi")?;
        let device = api
            .open_path(&std::ffi::CString::new(path)?)
            .context("Failed to open HID device")?;
        Ok(Self {
            backend: HidBackend::Local(device),
        })
    }

    pub fn open_fresh_for(device: &crate::device::Device) -> Result<Self> {
        #[cfg(target_os = "windows")]
        {
            return Self::open_proxy_for(device);
        }

        #[cfg(not(target_os = "windows"))]
        {
            Self::open_fresh_for_local(device)
        }
    }

    fn open_fresh_for_local(device: &crate::device::Device) -> Result<Self> {
        let mut last_error = None;
        for attempt in 0..10 {
            match Self::try_open_fresh_for(device) {
                Ok(device) => return Ok(device),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < 9 {
                        std::thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("unable to open the device")))
    }

    #[cfg(target_os = "windows")]
    fn open_proxy_for(device: &crate::device::Device) -> Result<Self> {
        let exe = std::env::current_exe().context("Failed to find Entropy executable")?;
        let device_json =
            serde_json::to_string(device).context("Failed to serialize HID device")?;
        let mut child = Command::new(exe)
            .arg("--entropy-hid-proxy")
            .arg(device_json)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start HID helper")?;

        let stdin = child.stdin.take().context("HID helper stdin unavailable")?;
        let stdout = child
            .stdout
            .take()
            .context("HID helper stdout unavailable")?;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(line) => {
                        if tx.send(line).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let ready_line = match rx.recv_timeout(Duration::from_secs(12)) {
            Ok(line) => line,
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                bail!("HID helper timed out while opening device");
            }
        };
        let ready: ProxyResponse = serde_json::from_str(&ready_line)
            .context("HID helper returned malformed startup response")?;
        if !ready.ok {
            let _ = child.kill();
            let _ = child.wait();
            bail!(ready
                .error
                .unwrap_or_else(|| "HID helper failed to open device".to_owned()));
        }

        Ok(Self {
            backend: HidBackend::Proxy(HidProxy {
                child: Mutex::new(child),
                stdin: Mutex::new(stdin),
                rx: Mutex::new(rx),
            }),
        })
    }

    fn try_open_fresh_for(device: &crate::device::Device) -> Result<Self> {
        let api = hidapi::HidApi::new().context("Failed to init hidapi")?;

        for info in api.device_list() {
            if !device_info_matches(info, device, true) {
                continue;
            }
            return info
                .open_device(&api)
                .map(|device| Self {
                    backend: HidBackend::Local(device),
                })
                .context("Failed to open HID device");
        }

        for info in api.device_list() {
            if !device_info_matches(info, device, false) {
                continue;
            }
            return info
                .open_device(&api)
                .map(|device| Self {
                    backend: HidBackend::Local(device),
                })
                .context("Failed to open HID device");
        }

        anyhow::bail!("HID device disappeared during reconnect")
    }

    /// Send exactly MSG_LEN bytes (with 0x00 report ID prepended), receive MSG_LEN bytes back.
    fn usb_send(&self, data: &[u8]) -> Result<[u8; MSG_LEN]> {
        match &self.backend {
            HidBackend::Local(device) => usb_send_local(device, data),
            #[cfg(target_os = "windows")]
            HidBackend::Proxy(proxy) => proxy.usb_send(data),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn usb_send_local(device: &hidapi::HidDevice, data: &[u8]) -> Result<[u8; MSG_LEN]> {
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

    let bytes_written = device.write(&write_buf).context("HID write failed")?;
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
    let bytes_read = device
        .read_timeout(&mut read_buf, 500)
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

#[cfg(target_os = "windows")]
impl HidProxy {
    fn usb_send(&self, data: &[u8]) -> Result<[u8; MSG_LEN]> {
        if data.len() > MSG_LEN {
            bail!(
                "HID command too long — {} bytes, max {} bytes",
                data.len(),
                MSG_LEN
            );
        }

        {
            let mut stdin = self
                .stdin
                .lock()
                .map_err(|_| anyhow::anyhow!("HID helper stdin lock poisoned"))?;
            writeln!(stdin, "{}", bytes_to_hex(data))
                .context("Failed to write HID helper request")?;
            stdin
                .flush()
                .context("Failed to flush HID helper request")?;
        }

        let line = {
            let rx = self
                .rx
                .lock()
                .map_err(|_| anyhow::anyhow!("HID helper receiver lock poisoned"))?;
            rx.recv_timeout(Duration::from_secs(2))
                .context("HID helper timed out during command")?
        };
        let response: ProxyResponse =
            serde_json::from_str(&line).context("HID helper returned malformed response")?;
        if !response.ok {
            if let Ok(mut child) = self.child.lock() {
                let _ = child.kill();
                let _ = child.wait();
            }
            bail!(response
                .error
                .unwrap_or_else(|| "HID helper command failed".to_owned()));
        }

        let bytes = hex_to_bytes(
            response
                .data
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("HID helper response missing data"))?,
        )?;
        if bytes.len() != MSG_LEN {
            bail!(
                "HID helper invalid response length — {} bytes, expected {}",
                bytes.len(),
                MSG_LEN
            );
        }
        let mut out = [0u8; MSG_LEN];
        out.copy_from_slice(&bytes);
        Ok(out)
    }
}

#[cfg(target_os = "windows")]
pub fn run_hid_proxy_if_requested() -> bool {
    let mut args = std::env::args();
    let _exe = args.next();
    if args.next().as_deref() != Some("--entropy-hid-proxy") {
        return false;
    }

    let result = (|| -> Result<()> {
        let device_json = args
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing HID helper device argument"))?;
        let device: crate::device::Device = serde_json::from_str(&device_json)
            .context("Failed to parse HID helper device argument")?;
        run_hid_proxy(device)
    })();

    if let Err(e) = result {
        let response = serde_json::to_string(&ProxyResponse {
            ok: false,
            data: None,
            error: Some(e.to_string()),
        })
        .unwrap_or_else(|_| {
            "{\"ok\":false,\"data\":null,\"error\":\"HID helper failed\"}".to_owned()
        });
        let _ = writeln!(std::io::stdout(), "{}", response);
        let _ = std::io::stdout().flush();
    }
    true
}

#[cfg(target_os = "windows")]
fn run_hid_proxy(device: crate::device::Device) -> Result<()> {
    let hid = HidDevice::open_fresh_for_local(&device)?;
    writeln!(
        std::io::stdout(),
        "{}",
        serde_json::to_string(&ProxyResponse {
            ok: true,
            data: None,
            error: None,
        })?
    )?;
    std::io::stdout().flush()?;

    for line in BufReader::new(std::io::stdin()).lines() {
        let line = line?;
        let response = match hex_to_bytes(line.trim()).and_then(|data| hid.usb_send(&data)) {
            Ok(data) => ProxyResponse {
                ok: true,
                data: Some(bytes_to_hex(&data)),
                error: None,
            },
            Err(e) => ProxyResponse {
                ok: false,
                data: None,
                error: Some(e.to_string()),
            },
        };
        writeln!(std::io::stdout(), "{}", serde_json::to_string(&response)?)?;
        std::io::stdout().flush()?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn bytes_to_hex(data: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(data.len() * 2);
    for &byte in data {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0F) as usize] as char);
    }
    out
}

#[cfg(target_os = "windows")]
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        bail!("invalid hex length");
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    let bytes = hex.as_bytes();
    for pair in bytes.chunks_exact(2) {
        let high = hex_nibble(pair[0])?;
        let low = hex_nibble(pair[1])?;
        out.push((high << 4) | low);
    }
    Ok(out)
}

#[cfg(target_os = "windows")]
fn hex_nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => bail!("invalid hex digit"),
    }
}
