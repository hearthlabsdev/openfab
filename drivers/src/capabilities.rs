//! This module provide structures for the driver to advertise the printers capabilities.
use serde::{Serialize, Deserialize};

pub mod capabilities {

    pub const REQUIRED_CAPABILITIES: &[&str] = &[
        PRINTER_INFO,
        PRINTER_STATUS,
    ];

    // Printer / machine
    pub const PRINTER_INFO: &str = "printer.info";
    pub const PRINTER_STATUS: &str = "printer.status";
    pub const PRINTER_IDENTIFY: &str = "printer.identify";

    // Print lifecycle
    pub const PRINT_STATUS: &str = "print.status";
    pub const PRINT_START: &str = "print.start";
    pub const PRINT_PAUSE: &str = "print.pause";
    pub const PRINT_RESUME: &str = "print.resume";
    pub const PRINT_STOP: &str = "print.stop";
    pub const PRINT_CANCEL: &str = "print.cancel";

    // G-code / print execution
    pub const PRINT_GCODE_STATUS: &str = "print.gcode.status";
    pub const PRINT_GCODE_CURRENT_LINE: &str = "print.gcode.current_line";
    pub const PRINT_GCODE_SEND: &str = "print.gcode.send";

    // Files
    pub const FILES_INDEX: &str = "files.index";
    pub const FILES_UPLOAD: &str = "files.upload";
    pub const FILES_DOWNLOAD: &str = "files.download";
    pub const FILES_DELETE: &str = "files.delete";
    pub const FILES_SYNC: &str = "files.sync";

    // Camera
    pub const CAMERA_CAPTURE: &str = "camera.capture";
    pub const CAMERA_FRAME: &str = "camera.frame";
    pub const CAMERA_STREAM: &str = "camera.stream";

    // Temperature
    pub const TEMPERATURE_STATUS: &str = "temperature.status";
    pub const TEMPERATURE_SET: &str = "temperature.set";

    // Fans
    pub const FAN_STATUS: &str = "fan.status";
    pub const FAN_SET: &str = "fan.set";

    // Motion
    pub const MOTION_STATUS: &str = "motion.status";
    pub const MOTION_HOME: &str = "motion.home";

    // Printer controls
    pub const PRINTER_RESET: &str = "printer.reset";
    pub const PRINTER_RESTART: &str = "printer.restart";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverCapabilities {
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
}
