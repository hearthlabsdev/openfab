use serialport::SerialPortInfo;

/// ABI-safe enum (guaranteed u32 layout)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialPortType {
    Unknown = 0,
    USB = 1,
    PCI = 2,
    Bluetooth = 3,
}

impl From<serialport::SerialPortType> for SerialPortType {
    fn from(port_type: serialport::SerialPortType) -> SerialPortType {
        match port_type {
            serialport::SerialPortType::UsbPort(_) => SerialPortType::USB,
            serialport::SerialPortType::PciPort => SerialPortType::PCI,
            serialport::SerialPortType::BluetoothPort => SerialPortType::Bluetooth,
            serialport::SerialPortType::Unknown => SerialPortType::Unknown,
        }
    }
}

pub struct PortInfo {
    pub name: String,
    pub port_type: SerialPortType,
}

impl From<SerialPortInfo> for PortInfo {
    fn from(port_info: SerialPortInfo) -> PortInfo {
        PortInfo {
            name: port_info.port_name,
            port_type: port_info.port_type.into(),
        }
    }
}

pub struct DeviceSerialPort {
    interface: Box<dyn serialport::SerialPort>,
}

impl DeviceSerialPort {
    pub fn list() -> Result<Vec<PortInfo>, std::io::Error> {
        Ok(
            serialport::available_ports()?
                .into_iter()
                .map(|v| v.into())
                .collect()
        )
    }
}