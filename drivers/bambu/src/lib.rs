use bambu_rs::client::LanMqttClient;
use bambu_rs::config::ResolvedTarget;
use bambu_rs::core::model::Model;
use openfab_drivers::capabilities::DriverCapabilities;
use openfab_drivers::config::{
    ConfigField, ConfigSchema, DriverMeta
};

use openfab_drivers::errors::DriverError;
use openfab_drivers::utils::get_value_type;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::io::AsyncRead;
pub struct BambuMeta;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientTarget {
    ip: String,
    access_code: String,
    model: String,
    serial: String,
    mqtt_port: u16,
    camera_port: u16,
    ftps_port: u16,
    detect_port: u16,
}

impl ClientTarget {
    pub fn resolve(self) -> ResolvedTarget {
        ResolvedTarget {
            ip: self.ip,
            access_code: self.access_code,
            model: Model::from_config_str(&self.model),
            serial: self.serial,
            mqtt_port: self.mqtt_port,
            camera_port: self.camera_port,
            ftps_port: self.ftps_port,
            detect_port: self.detect_port,
        }
    }
}

impl From<ClientTarget> for ResolvedTarget {
    fn from(target: ClientTarget) -> ResolvedTarget {
        target.resolve()
    }
}

pub struct BambuDriver {
    // identified by serial number?
    clients: HashMap<String, LanMqttClient>,
}

impl BambuDriver {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn metadata() -> DriverMeta {
        DriverMeta {
            id: "bambu-lab-lan".into(),
            name: "Bambu Lab".into(),
            version: "0.1.0".into(),
            schema: ConfigSchema {
                fields: vec![
                    ConfigField {
                        name: "ip".to_string(),
                        field_type: "text".to_string(),
                        label: "IP Address or Hostname".to_string(),
                        default: None,
                        required: Some(true),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "access_code".to_string(),
                        field_type: "password".to_string(),
                        label: "access code".to_string(),
                        default: None,
                        required: Some(true),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "model".to_string(),
                        field_type: "text".to_string(),
                        label: "model".to_string(),
                        default: None,
                        required: Some(true),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "serial".to_string(),
                        field_type: "text".to_string(),
                        label: "Serial Number".to_string(),
                        default: None,
                        required: Some(true),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "mqtt_port".to_string(),
                        field_type: "number".to_string(),
                        label: "MQTT Port".to_string(),
                        default: Some("8883".to_string()),
                        required: Some(true),
                        min: Some(1),
                        max: Some(65535),
                    },
                    ConfigField {
                        name: "ftps_port".to_string(),
                        field_type: "number".to_string(),
                        label: "FTPS Port".to_string(),
                        default: Some("21".to_string()),
                        required: Some(true),
                        min: Some(1),
                        max: Some(65535),
                    },
                    ConfigField {
                        name: "camera_port".to_string(),
                        field_type: "number".to_string(),
                        label: "Camera Port".to_string(),
                        default: Some("322".to_string()),
                        required: Some(true),
                        min: Some(1),
                        max: Some(65535),
                    },
                    ConfigField {
                        name: "detect_port".to_string(),
                        field_type: "number".to_string(),
                        label: "Detect Port".to_string(),
                        default: Some("1990".to_string()),
                        required: Some(true),
                        min: Some(1),
                        max: Some(65535),
                    },
                ],
            },
        }
    }
}
