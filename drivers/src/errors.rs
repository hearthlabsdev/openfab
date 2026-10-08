use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("configuration error: {0}")]
    Configuration(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("device error: {0}")]
    Device(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),

    #[error("couldn't find driver")]
    DriverNotFound,

    #[error("configuration wasn't the expected type expect: {0} found {1}")]
    InvalidConfigType(String, String),

    #[error("missing a required configuration option: {0}")]
    MissingConfigValue(String),

    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("error occured in driver runtime")]
    Runtime(String),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required configuration field: {0}")]
    MissingField(String),

    #[error("invalid value for configuration field '{0}'")]
    InvalidField(String),

    #[error("configuration field '{0}' is below the minimum value")]
    BelowMinimum(String),

    #[error("configuration field '{0}' is above the maximum value")]
    AboveMaximum(String),

    #[error("unknown configuration field: {0}")]
    UnknownField(String),

    #[error("unknown configuration field type: {0}")]
    UnknownFieldType(String),

    #[error("not an object")]
    InvalidConfig,
}

/// serialization and deserialization may be required to pass the wasm boundary in the future.
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum PrintError {}