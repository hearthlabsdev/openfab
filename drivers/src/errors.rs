//! # Error Handling and Failure Types Module
//!
//! This module defines unified error enums used throughout the driver SDK. 
//! 
//! By centralizing error definitions into categories like [`DriverError`], [`ConfigError`], 
//! and [`StatusErr`], downstream makerspace applications and UI dashboards can cleanly handle 
//! communication drops, missing telemetry fields, and configuration mismatches without needing 
//! to parse unstructured strings.

use crate::status::Extensions;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use serde_json::Value;

/// Core error types that can occur during driver operations, communications, or parsing.
#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    /// An error occurred while validating or parsing driver configuration.
    #[error("configuration error: {0}")]
    Configuration(String),

    /// Failed to connect to or maintain communication with the hardware backend.
    #[error("connection error: {0}")]
    Connection(String),

    /// A hardware-level error returned by the device.
    #[error("device error: {0}")]
    Device(String),

    /// The specified device identifier could not be located in the driver registry.
    #[error("device not found: {0}")]
    DeviceNotFound(String),

    /// The requested operation is not supported by this specific hardware backend or driver.
    #[error("unsupported operation: {0}")]
    Unsupported(String),

    /// The requested driver backend could not be found.
    #[error("couldn't find driver")]
    DriverNotFound,

    /// A configuration value did not match the expected type schema.
    /// Carries the expected type description and the found type description.
    #[error("configuration wasn't the expected type expect: {0} found {1}")]
    InvalidConfigType(String, String),

    /// A mandatory configuration option was omitted.
    #[error("missing a required configuration option: {0}")]
    MissingConfigValue(String),

    /// An error occurred while serializing or deserializing JSON payloads.
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// An unrecoverable error occurred within the background driver runtime worker.
    #[error("error occured in driver runtime")]
    Runtime(String),

    /// Failed to parse or extract core printer status telemetry.
    #[error("failed to get status: {0}")]
    StatusErr(#[from] StatusErr),

    /// A custom or backend-specific error message.
    #[error("{0}")]
    Custom(String),
}

/// Errors related to configuration schema validation, missing fields, or out-of-bounds values.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// A mandatory configuration field was left empty or omitted.
    #[error("missing required configuration field: {0}")]
    MissingField(String),

    /// The value provided for a configuration field is malformed or invalid.
    #[error("invalid value for configuration field '{0}'")]
    InvalidField(String),

    /// A numerical configuration field fell below its permitted minimum boundary.
    #[error("configuration field '{0}' is below the minimum value")]
    BelowMinimum(String),

    /// A numerical configuration field exceeded its permitted maximum boundary.
    #[error("configuration field '{0}' is above the maximum value")]
    AboveMaximum(String),

    /// An unrecognized configuration field key was supplied.
    #[error("unknown configuration field: {0}")]
    UnknownField(String),

    /// The declared type of a configuration field is unknown or unsupported.
    #[error("unknown configuration field type: {0}")]
    UnknownFieldType(String),

    /// The configuration payload root is malformed and expected a JSON object.
    #[error("not an object")]
    InvalidConfig,
}

/// Errors raised when parsing essential temperature or sensor telemetry fails.
#[derive(Debug, Error)]
pub enum StatusErr {
    /// The required heated bed temperature telemetry was missing from the payload.
    #[error("missing bed temperature")]
    MissingBedTemp,
    /// The required print nozzle temperature telemetry was missing from the payload.
    #[error("missing nozzle temperature")]
    MissingNozzleTemp,
    #[error("failed to serialize extras: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum JobStatusErr {
    #[error("missing print job time remaining estimate")]
    MissingTimeRemaining,
    #[error("missing print job progress")]
    MissingProgress,
    #[error("missing job state")]
    MissingJobState,
    #[error("failed to serialize extras: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Specialized print job failure errors.
///
/// Note: Serialization and deserialization derive attributes are included to allow 
/// these errors to cross WebAssembly (WASM) boundaries safely in future front-end integrations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintError {
    pub code: String,
    pub message: Option<String>,
    pub url: Option<String>,
    extras: HashMap<String, Value>,
}

impl PrintError {
    pub fn with_extras<E: Extensions>(
        code: String,
        message: Option<String>,
        url: Option<String>,
        extra: E,
    ) -> Result<Self, serde_json::Error> {
        let val = serde_json::to_value(extra)?;
        let mut extras: HashMap<String, Value> = serde_json::from_value(val)?;
        extras.retain(|_, v| !v.is_null());
        Ok(Self {
            code,
            message,
            url,
            extras,
        })
    }
}