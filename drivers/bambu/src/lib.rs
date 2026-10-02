use openfab_drivers::config::{ConfigSchema, ConfigField, DriverMetadata};

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
