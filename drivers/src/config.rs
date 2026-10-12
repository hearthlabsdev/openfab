//! # Configuration and Metadata Management Module
//!
//! This module defines driver metadata schemas, configuration property definitions, and validation fields
//! used by the SDK registry and user interfaces to dynamically construct configuration forms for each hardware backend.

use crate::errors::DriverError;
use serde::{Deserialize, Serialize};

/// A type alias for driver operation results returning [`DriverError`].
pub type DriverResult<T> = Result<T, DriverError>;

/// Defines an individual configuration field exposed by a driver schema (e.g., IP address, API key, serial number).
///
/// Used by makerspace dashboards and administrative UIs to auto-generate setup forms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    /// The unique identifier or key for the configuration parameter.
    pub name: String,
    /// The data type of the field (e.g., `"string"`, `"integer"`, `"boolean"`).
    pub field_type: String,
    /// A human-readable label displayed in configuration UIs.
    pub label: String,
    /// Optional default value if none is provided by the user.
    pub default: Option<String>,
    /// Specifies whether this configuration field is mandatory.
    pub required: Option<bool>,
    /// Minimum numerical value constraint (if applicable).
    pub min: Option<u32>,
    /// Maximum numerical value constraint (if applicable).
    pub max: Option<u32>,
}

/// A collection of configuration fields required to initialize a specific driver backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// List of configuration fields required by the driver.
    pub fields: Vec<ConfigField>,
}

/// Metadata describing a specific driver implementation and its configuration requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverMeta {
    /// Unique driver identifier string (e.g., `"prusa-link"` or `"bambu-lan"`).
    pub id: String,
    /// Human-readable name of the driver backend.
    pub name: String,
    /// Semantic version string of the driver.
    pub version: String,
    /// The configuration schema required to instantiate this driver.
    pub schema: ConfigSchema,
}

impl DriverMeta {
    /// Returns the unique identifier of the driver.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the human-readable name of the driver.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the semantic version of the driver.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns a reference to the driver's configuration schema.
    pub fn config_schema(&self) -> &ConfigSchema {
        &self.schema
    }
}
