//! A module that contains all the errors used by this crate.

use thiserror::Error;
use secret_ref::RefParseError;

/// Top-level error returned by printing / runtime operations.
#[derive(Debug, Error)]
pub enum PrintErr {
    /// Failure related to configuration loading or validation.
    #[error("configuration failure: {0}")]
    ConfigErr(#[from] ConfigError),
}

/// Errors that can occur while assembling the top-level configuration.
#[derive(Debug, thiserror::Error)]
pub enum SetupConfigError {
    /// The server configuration section is missing.
    #[error("server configuration is missing")]
    MissingServer,

    /// The database configuration section is missing.
    #[error("database configuration is missing")]
    MissingDatabase,

    /// An API endpoint with the same name already exists.
    #[error("endpoint '{0}' already exists")]
    DuplicateEndpoint(String),

    /// No OIDC providers were configured.
    #[error("missing OIDC providers")]
    MissingOIDCProviders,
}

/// Errors related to validating the database configuration.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseConfigError {
    /// The database name was not provided.
    #[error("database name cannot be empty")]
    MissingDatabase,

    /// The database user was not provided.
    #[error("user cannot be empty")]
    MissingUser,

    /// The database host is missing.
    #[error("missing host")]
    MissingHost,

    /// The database port is missing.
    #[error("missing port")]
    MissingPort,

    /// The configured port is invalid (zero).
    #[error("port must be non-zero")]
    InvalidPort,

    /// A password reference/file was not provided.
    #[error("missing password file reference")]
    MissingPasswd,

    /// The database driver (e.g. postgres, mysql) was not specified.
    #[error("missing database driver eg. postgres, mysql, mariadb, etc")]
    MissingDriver,
}

/// Errors related to validating server configuration.
#[derive(Debug, thiserror::Error)]
pub enum ServerConfigError {
    /// TLS is enabled but no certificate/key or self-signed configuration was provided.
    #[error("SSL is enabled but neither cert/key nor self-signed is configured")]
    MissingTlsMaterial,

    /// Both self-signed TLS and certificate/key configuration were specified.
    #[error("Both self-signed and cert/key were specified")]
    ConflictingTlsConfig,

    /// Only one of `cert_file` or `key_file` was provided.
    #[error("cert_file requires key_file (and vice versa)")]
    IncompleteTlsFiles,

    /// The server port was not specified.
    #[error("port must be specified")]
    MissingPort,

    /// The server hostname is missing.
    #[error("missing hostname")]
    MissingHost,

    /// The IP address to bind to was not provided.
    #[error("missing IP address to bind to")]
    MissingAddr,
}

/// Errors produced while parsing or validating the full configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Server configuration validation failed.
    #[error("server configuration error: {0}")]
    Server(#[from] ServerConfigError),

    /// Database configuration validation failed.
    #[error("database configuration error: {0}")]
    Database(#[from] DatabaseConfigError),

    /// A required field is missing from a configuration section.
    #[error("missing field {0} on {1}")]
    MissingField(&'static str, &'static str),

    /// Failed to parse a network address.
    #[error("failed to parse address because of: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    /// Failed to parse a secret reference.
    #[error("failed to parse secret ref: {0}")]
    SecretRefParseError(#[from] RefParseError),

    /// Failed while retrieving a secret.
    #[error("failed to fetch secret")]
    SecretErr(#[from] secret_ref::SecretError),
}

/// Errors related to API endpoint configuration.
#[derive(Debug, thiserror::Error)]
pub enum ApiEndpointError {
    /// The endpoint port was invalid (zero).
    #[error("port must be non-zero")]
    InvalidPort,
}