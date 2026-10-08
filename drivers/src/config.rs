use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::io::AsyncRead;
use crate::capabilities::DriverCapabilities;
use crate::errors::DriverError;


pub type DriverResult<T> = Result<T, DriverError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub name: String,
    pub field_type: String,
    pub label: String,
    pub default: Option<String>,
    pub required: Option<bool>,
    pub min: Option<u32>,
    pub max: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    pub fields: Vec<ConfigField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub schema: ConfigSchema,
}

impl DriverMeta {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn config_schema(&self) -> &ConfigSchema {
        &self.schema
    }
}
/*
impl<T: DriverMetadata> From<T> for DriverMeta {
    fn from(m: T) -> DriverMeta {
        DriverMeta {
            id: m.id().to_string(),
            name: m.name().to_string(),
            version: m.version().to_string(),
            schema: m.config_schema(),
        }
    }
}

impl From<&dyn DriverMetadata> for DriverMeta {
    fn from(m: &dyn DriverMetadata) -> DriverMeta {
        DriverMeta {
            id: m.id().to_string(),
            name: m.name().to_string(),
            version: m.version().to_string(),
            schema: m.config_schema(),
        }
    }
}

/// Information exposed by a driver to the registry/UI.
pub trait DriverMetadata {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;

    /// JSON Schema describing the configuration this driver accepts.
    fn config_schema(&self) -> ConfigSchema;
}


/// A configured instance of a device driver.
#[async_trait]
pub trait DeviceDriver: Send + Sync {
    fn metadata(&self) -> &dyn DriverMetadata;

    /// Connect using configuration supplied by the user.
    async fn connect(&mut self, config: Value) -> DriverResult<()>;

    async fn disconnect(&mut self) -> DriverResult<()>;

    async fn status(&self) -> DriverResult<DeviceStatus>;

    /// Generic capability discovery.
    fn capabilities(&self) -> DriverCapabilities;

    async fn start_print(&self, device: &str, file: Box<dyn AsyncRead + Send + 'static>) -> Result<(), DriverError>;

    /// Optional generic command interface.
    async fn command(
        &mut self,
        command: &str,
        parameters: HashMap<String, Value>,
    ) -> DriverResult<Value>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceStatus {
    Disconnected,
    Connecting,
    Ready,
    Busy,
    Error { message: String },
}
*/