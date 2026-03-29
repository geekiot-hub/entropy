use crate::firmware::FirmwareProtocol;

/// Represents a connected keyboard device (Vial/HID or ZMK/Serial).
#[derive(Debug, Clone)]
pub struct Device {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    /// HID path (Vial) or serial port name (ZMK)
    pub path: String,
    pub firmware: FirmwareProtocol,
}

/// Scans for connected keyboard devices (both Vial HID and ZMK serial).
pub struct DeviceManager {
    devices: Vec<Device>,
}

impl DeviceManager {
    pub fn new() -> Self {
        let mut mgr = Self { devices: vec![] };
        mgr.scan();
        mgr
    }

    pub fn scan(&mut self) {
        self.devices.clear();

        #[cfg(not(target_arch = "wasm32"))]
        {
            // --- Vial/HID devices ---
            if let Ok(api) = hidapi::HidApi::new() {
                for info in api.device_list() {
                    // Filter: vial usage page 0xFF60, usage 0x61
                    if info.usage_page() == 0xFF60 && info.usage() == 0x61 {
                        self.devices.push(Device {
                            name: info
                                .product_string()
                                .unwrap_or("Unknown Keyboard")
                                .to_string(),
                            vendor_id: info.vendor_id(),
                            product_id: info.product_id(),
                            path: info.path().to_string_lossy().to_string(),
                            firmware: FirmwareProtocol::Vial,
                        });
                    }
                }
            }

            // --- ZMK/Serial devices ---
            if let Ok(ports) = serialport::available_ports() {
                for port in ports {
                    let is_zmk = match &port.port_type {
                        serialport::SerialPortType::UsbPort(info) => {
                            let product = info.product.as_deref().unwrap_or("").to_lowercase();
                            let manufacturer = info.manufacturer.as_deref().unwrap_or("").to_lowercase();
                            // Known ZMK VIDs: 0x1D50 (OpenMoko/generic ZMK)
                            let zmk_vid = info.vid == 0x1D50;
                            product.contains("zmk")
                                || manufacturer.contains("zmk")
                                || product.contains("studio")
                                || product.contains("ergohaven")
                                || manufacturer.contains("ergohaven")
                                || zmk_vid
                        }
                        _ => false,
                    };

                    if is_zmk {
                        let name = match &port.port_type {
                            serialport::SerialPortType::UsbPort(info) => {
                                info.product.clone().unwrap_or_else(|| port.port_name.clone())
                            }
                            _ => port.port_name.clone(),
                        };
                        self.devices.push(Device {
                            name: format!("{} (ZMK)", name),
                            vendor_id: match &port.port_type {
                                serialport::SerialPortType::UsbPort(i) => i.vid,
                                _ => 0,
                            },
                            product_id: match &port.port_type {
                                serialport::SerialPortType::UsbPort(i) => i.pid,
                                _ => 0,
                            },
                            path: port.port_name.clone(),
                            firmware: FirmwareProtocol::Zmk,
                        });
                    }
                }
            }
        }

        log::info!("Found {} device(s)", self.devices.len());
    }

    pub fn devices(&self) -> &[Device] {
        &self.devices
    }
}
