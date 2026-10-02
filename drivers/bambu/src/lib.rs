use openfab_drivers::config::{ConfigSchema, DeviceStatus, DeviceDriver, ConfigField, DriverMetadata};
use openfab_drivers::errors::{DriverError};
use openfab_drivers::capabilities::{DriverCapabilities};
use serde_json::Value;
use std::collections::HashMap;

use bambu_rs::client::LanMqttClient;
pub struct BambuMeta;

impl DriverMetadata for BambuMeta {
    fn id(&self) -> &'static str {
        "bambu-lab-lan"
    }
    fn name(&self) -> &'static str {
        "Bambu Lab"
    }
    fn version(&self) -> &'static str {
        "0.1.0"
    }

    /// JSON Schema describing the configuration this driver accepts.
    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema {
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
                    name: "mqtt_port".to_string(),
                    field_type: "number".to_string(),
                    label: "MQTT Port".to_string(),
                    default: Some("8883".to_string()),
                    required: Some(true),
                    min: Some(1),
                    max: Some(65535),
                },
            ],
        }
    }
}

pub struct BambuDriver {
    client: LanMqttClient,
}

#[async_trait::async_trait]
impl DeviceDriver for BambuDriver {
    fn metadata(&self) -> &dyn DriverMetadata {
        &BambuMeta {}
    }

    /// Connect using configuration supplied by the user.
    async fn connect(&mut self, config: Value) -> Result<(), DriverError> {
        unimplemented!();
    }

    async fn disconnect(&mut self) -> Result<(), DriverError> {
        unimplemented!();
    }

    async fn status(&self) -> Result<DeviceStatus, DriverError> {
        unimplemented!();
    }

    /// Generic capability discovery.
    fn capabilities(&self) -> DriverCapabilities {
        unimplemented!();
    }

    /// Optional generic command interface.
    async fn command(
        &mut self,
        command: &str,
        parameters: HashMap<String, Value>,
    ) -> Result<Value, DriverError> {
        unimplemented!();
    }
}