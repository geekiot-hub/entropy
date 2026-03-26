/// Represents a connected HID keyboard device.
#[derive(Debug, Clone)]
pub struct Device {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub path: String,
}

/// Scans for connected HID devices (vial-compatible keyboards).
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
