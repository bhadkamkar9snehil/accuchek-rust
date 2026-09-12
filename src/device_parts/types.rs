
const USB_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_DATA_PAGES: usize = 2048;
const ROCHE_VENDOR_ID: u16 = 0x173a;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingRange {
    Normal,
    High,
    Low,
}

impl ReadingRange {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::High => "high",
            Self::Low => "low",
        }
    }
}

/// A blood glucose reading exactly as obtained from the meter plus a normalized analysis value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlucoseReading {
    /// Sequential index within this download. It is not used as a database identity.
    pub id: usize,
    /// Compatibility sort key derived from the meter-local wall clock. This is NOT UTC.
    pub epoch: i64,
    /// Meter-local wall-clock timestamp in ISO-like form, with no implied timezone.
    pub timestamp: String,
    #[serde(rename = "mg/dL")]
    pub mg_dl: u16,
    #[serde(rename = "mmol/L")]
    pub mmol_l: f64,
    /// Unmodified 16-bit measurement value from the device.
    pub raw_value: u16,
    /// Unmodified IEEE 11073 measurement-status word. Never silently discard it.
    pub status: u16,
    pub range_state: ReadingRange,
    /// Stable device identity used for local deduplication where available.
    pub device_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceMetadata {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: String,
    pub usb_product: String,
    pub usb_serial: Option<String>,
    pub model_number: Option<u16>,
    pub model_name: String,
    pub serial_number: Option<String>,
    pub meter_time: Option<String>,
    pub device_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResult {
    pub device: DeviceMetadata,
    pub readings: Vec<GlucoseReading>,
}

/// Represents a matched Roche USB interface.
#[derive(Debug)]
pub struct AccuChekDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub vendor: String,
    pub product: String,
    pub usb_serial: Option<String>,
    pub bus_number: u8,
    pub device_address: u8,
    pub config_value: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub send_endpoint: u8,
    pub receive_endpoint: u8,
}

impl AccuChekDevice {
    pub fn show(&self, msg: &str) {
        info!(
            "{} bus={} address={} config={} interface={} alt={} vendor=0x{:04x} product=0x{:04x} out=0x{:02x} in=0x{:02x}",
            msg,
            self.bus_number,
            self.device_address,
            self.config_value,
            self.interface_number,
            self.alternate_setting,
            self.vendor_id,
            self.product_id,
            self.send_endpoint,
            self.receive_endpoint,
        );
    }
}

fn protocol_error(context: &str, detail: impl std::fmt::Display) -> AccuChekError {
    AccuChekError::Protocol(format!("{}: {}", context, detail))
}

fn require_len(buffer: &[u8], minimum: usize, context: &str) -> Result<(), AccuChekError> {
    if buffer.len() < minimum {
        return Err(protocol_error(
            context,
            format!("truncated packet: got {} bytes, need at least {}", buffer.len(), minimum),
        ));
    }
    Ok(())
}

fn u16_at(buffer: &[u8], offset: usize, context: &str) -> Result<u16, AccuChekError> {
    require_len(buffer, offset + 2, context)?;
    Ok(u16::from_be_bytes([buffer[offset], buffer[offset + 1]]))
}

fn u32_at(buffer: &[u8], offset: usize, context: &str) -> Result<u32, AccuChekError> {
    require_len(buffer, offset + 4, context)?;
    Ok(u32::from_be_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ]))
}

fn optional_string<T: UsbContext>(
    handle: &DeviceHandle<T>,
    index: Option<u8>,
    fallback: &str,
) -> String {
    index
        .and_then(|i| handle.read_string_descriptor_ascii(i).ok())
        .filter(|s| !s.trim_matches('\0').trim().is_empty())
        .map(|s| s.trim_matches('\0').trim().to_string())
        .unwrap_or_else(|| fallback.to_string())
}

/// Match by explicit Roche VID/PID whitelist first, then locate a usable bulk IN/OUT interface.
fn check_device<T: UsbContext>(
    device: &rusb::Device<T>,
    config: &Config,
) -> Option<AccuChekDevice> {
    let desc = device.device_descriptor().ok()?;

    if desc.vendor_id() != ROCHE_VENDOR_ID
        || !config.is_device_valid(desc.vendor_id(), desc.product_id())
    {
        return None;
    }

    let mut selected = None;
    for config_index in 0..desc.num_configurations() {
        let cfg = device.config_descriptor(config_index).ok()?;
        for interface in cfg.interfaces() {
            for alt in interface.descriptors() {
                let mut in_endpoint = None;
                let mut out_endpoint = None;
                for endpoint in alt.endpoint_descriptors() {
                    if endpoint.transfer_type() != rusb::TransferType::Bulk {
                        continue;
                    }
                    match endpoint.direction() {
                        rusb::Direction::In if in_endpoint.is_none() => {
                            in_endpoint = Some(endpoint.address())
                        }
                        rusb::Direction::Out if out_endpoint.is_none() => {
                            out_endpoint = Some(endpoint.address())
                        }
                        _ => {}
                    }
                }
                if let (Some(receive_endpoint), Some(send_endpoint)) = (in_endpoint, out_endpoint) {
                    selected = Some((
                        cfg.number(),
                        alt.interface_number(),
                        alt.setting_number(),
                        send_endpoint,
                        receive_endpoint,
                    ));
                    break;
                }
            }
            if selected.is_some() {
                break;
            }
        }
        if selected.is_some() {
            break;
        }
    }

    let (config_value, interface_number, alternate_setting, send_endpoint, receive_endpoint) =
        selected?;

    let handle = device.open().ok()?;
    let vendor = optional_string(&handle, desc.manufacturer_string_index(), "Roche");
    let product = optional_string(&handle, desc.product_string_index(), "Accu-Chek");
    let usb_serial = desc
        .serial_number_string_index()
        .and_then(|i| handle.read_string_descriptor_ascii(i).ok())
        .map(|s| s.trim_matches('\0').trim().to_string())
        .filter(|s| !s.is_empty());

    Some(AccuChekDevice {
        vendor_id: desc.vendor_id(),
        product_id: desc.product_id(),
        vendor,
        product,
        usb_serial,
        bus_number: device.bus_number(),
        device_address: device.address(),
        config_value,
        interface_number,
        alternate_setting,
        send_endpoint,
        receive_endpoint,
    })
}
