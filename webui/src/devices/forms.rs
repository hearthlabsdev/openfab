use rocket::form::{FromForm, FromFormField};
use serde::Deserialize;

#[derive(Debug, FromFormField, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    Network,
    Serial,
}

#[derive(Debug, FromFormField, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    // IPP Family
    Ipp,
    Ipps,
    Ipp3d,

    // Raw Network
    Jetdirect,
    Lpd,

    // Vendor APIs
    Prusalink,
    BambuLan,
    Moonraker,

    // Generic
    Http,
    Custom,

    // Serial Drivers
    Marlin,
    Klipper,
    GenericGcode,
}

#[derive(Debug, FromFormField, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Printer,
    Scanner,
    Copier,
}

#[derive(Debug, FromForm, Deserialize)]
pub struct DeviceUploadForm {
    // =========================
    // Core Identity
    // =========================
    pub name: String,

    // =========================
    // Connection Method
    // =========================
    pub connection_type: ConnectionType,

    pub ip_address: Option<String>,
    pub serial_port: Option<String>,

    // =========================
    // Protocol / Driver
    // =========================
    pub protocol: Protocol,

    // =========================
    // Optional Metadata
    // =========================
    pub device_type: Option<DeviceType>, // maps from "type"
    pub location: Option<String>,
    pub model: Option<String>,
    pub manufacturer: Option<String>,

    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
    pub mac_address: Option<String>,
    pub description: Option<String>,
}

impl DeviceUploadForm {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Device name is required".into());
        }

        match self.connection_type {
            ConnectionType::Network => {
                if self.ip_address.as_deref().unwrap_or("").is_empty() {
                    return Err("IP address is required for network devices".into());
                }
            }
            ConnectionType::Serial => {
                if self.serial_port.as_deref().unwrap_or("").is_empty() {
                    return Err("Serial port is required for serial devices".into());
                }
            }
        }

        Ok(())
    }
}