use crate::firmware::FirmwareProtocol;

/// Represents a connected Vial/HID keyboard device.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Device {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: String,
    pub serial_number: String,
    /// HID path used by Vial.
    pub path: String,
    pub firmware: FirmwareProtocol,
}

impl Device {
    pub fn is_likely_rmk(&self) -> bool {
        self.manufacturer.to_ascii_lowercase().contains("rmk")
            || self.name.to_ascii_lowercase().contains("rmk")
            || self.serial_number.to_ascii_lowercase().contains("rmk")
    }
}

/// Scans for connected Vial HID keyboard devices.
pub struct DeviceManager {
    devices: Vec<Device>,
}

impl DeviceManager {
    pub fn new() -> Self {
        let mut mgr = Self { devices: vec![] };
        mgr.scan();
        mgr
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn scan_devices() -> Vec<Device> {
        let mut devices = Vec::new();

        if let Ok(api) = hidapi::HidApi::new() {
            for info in api.device_list() {
                // Filter: Vial usage page 0xFF60, usage 0x61
                if info.usage_page() == 0xFF60 && info.usage() == 0x61 {
                    devices.push(Device {
                        name: info
                            .product_string()
                            .unwrap_or("Unknown Keyboard")
                            .to_string(),
                        vendor_id: info.vendor_id(),
                        product_id: info.product_id(),
                        manufacturer: info.manufacturer_string().unwrap_or("").to_string(),
                        serial_number: info.serial_number().unwrap_or("").to_string(),
                        path: info.path().to_string_lossy().to_string(),
                        firmware: FirmwareProtocol::Vial,
                    });
                }
            }
        }

        devices
    }

    pub fn scan(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.devices = Self::scan_devices();
        }

        log::info!("Found {} Vial device(s)", self.devices.len());
    }

    pub fn replace_devices(&mut self, devices: Vec<Device>) {
        self.devices = devices;
        log::info!("Found {} Vial device(s)", self.devices.len());
    }

    pub fn devices(&self) -> &[Device] {
        &self.devices
    }
}
