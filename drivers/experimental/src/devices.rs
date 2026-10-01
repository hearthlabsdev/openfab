use crate::serial::DeviceSerialPort;

pub enum HardwareDevice {
    GPIO(),
    Serial(DeviceSerialPort),
}