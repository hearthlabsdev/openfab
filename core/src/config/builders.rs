//! Builders for constructing validated server and database configuration.

use crate::config::{DatabaseConfig, DatabaseDriver};
use crate::errors::{ServerConfigError, DatabaseConfigError};
use crate::config::{ServerConfig};

use std::path::PathBuf;
use std::net::IpAddr;

use secret_ref::SecretRef;

/// Builder for [`ServerConfig`] instances.
///
/// This builder allows gradual construction of server configuration
/// while performing validation during [`build`](Self::build).
///
/// Note: In production deployments, TLS certificate management is
/// typically handled by a reverse proxy such as Caddy or nginx.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ServerConfigBuilder {
    /// Directory containing configuration assets.
    pub config_dir: Option<PathBuf>,

    /// Optional path to a configuration file.
    pub config_file: Option<PathBuf>,

    /// Whether TLS/SSL is explicitly enabled.
    pub ssl: Option<bool>,

    /// The IP address the server should bind to.
    pub addr: Option<IpAddr>,

    /// Hostname used for reflection or external references.
    pub host: Option<String>,

    /// TCP port the server should listen on.
    pub port: Option<u16>,

    /// Path or reference to the TLS certificate file.
    pub cert_file: Option<SecretRef>,

    /// Path or reference to the TLS private key file.
    pub key_file: Option<SecretRef>,

    /// Whether a self-signed certificate should be generated.
    pub self_signed: Option<bool>,
}

impl ServerConfigBuilder {

    /// Creates a new empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the configuration directory.
    pub fn config_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.config_dir = Some(dir.into());
        self
    }

    /// Sets the configuration file path.
    pub fn config_file<P: Into<PathBuf>>(mut self, file: P) -> Self {
        self.config_file = Some(file.into());
        self
    }

    /// Enables or disables TLS/SSL.
    pub fn ssl(mut self, enabled: bool) -> Self {
        self.ssl = Some(enabled);
        self
    }

    /// Sets the IP address to bind the server to.
    pub fn addr(mut self, addr: IpAddr) -> Self {
        self.addr = Some(addr);
        self
    }

    /// Sets the hostname used by the server.
    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets the listening port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Sets the TLS certificate file reference.
    pub fn cert_file<P: Into<SecretRef>>(mut self, path: P) -> Self {
        self.cert_file = Some(path.into());
        self
    }

    /// Sets the TLS private key file reference.
    pub fn key_file<P: Into<SecretRef>>(mut self, path: P) -> Self {
        self.key_file = Some(path.into());
        self
    }

    /// Enables or disables self-signed certificate generation.
    pub fn self_signed(mut self, enabled: bool) -> Self {
        self.self_signed = Some(enabled);
        self
    }

    /// Returns whether TLS should be considered enabled.
    ///
    /// TLS is enabled if explicitly set or if TLS configuration
    /// elements such as certificates or self-signed mode are present.
    pub fn tls_enabled(&self) -> bool {
        match self.ssl {
            Some(v) => v,
            None => self.cert_file.is_some()
                || self.key_file.is_some()
                || self.self_signed.unwrap_or(false),
        }
    }

    /// Returns true if TLS is expected to be handled externally
    /// (e.g. by a reverse proxy).
    pub fn uses_external_tls(&self) -> bool {
        self.tls_enabled()
            && self.cert_file.is_none()
            && self.key_file.is_none()
            && !self.self_signed.unwrap_or(false)
    }

    /// Returns whether the minimal server configuration fields
    /// have been provided.
    pub fn is_setup(&self) -> bool {
        self.host.is_some() && self.addr.is_some() && self.port.is_some()
    }

    /// Validates and constructs a [`ServerConfig`].
    ///
    /// This checks TLS configuration consistency and ensures
    /// required fields are present.
    pub fn build(self) -> Result<ServerConfig, ServerConfigError> {

        let cert = self.cert_file.is_some();
        let key = self.key_file.is_some();
        let self_signed = self.self_signed.unwrap_or(false);

        if cert ^ key {
            return Err(ServerConfigError::IncompleteTlsFiles);
        }

        if self_signed && cert {
            return Err(ServerConfigError::ConflictingTlsConfig);
        }

        if self.tls_enabled() && !self_signed && !cert && !self.uses_external_tls() {
            return Err(ServerConfigError::MissingTlsMaterial);
        }

        Ok(ServerConfig {
            host: self.host.ok_or(ServerConfigError::MissingHost)?,
            addr: self.addr.ok_or(ServerConfigError::MissingAddr)?,
            port: self.port.ok_or(ServerConfigError::MissingPort)?,
            config_file: self.config_file,
            ssl: self.ssl,
            cert_file: self.cert_file,
            key_file: self.key_file,
            self_signed: self.self_signed,
        })
    }
}

/// Builder for [`DatabaseConfig`] values.
///
/// Provides validation and helper utilities for constructing
/// database connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfigBuilder {
    /// Database host name or address.
    pub host: Option<String>,

    /// Database port.
    pub port: Option<u16>,

    /// Database driver implementation.
    pub driver: Option<DatabaseDriver>,

    /// Database name.
    pub database: Option<String>,

    /// Database username.
    pub user: Option<String>,

    /// Secret reference for the database password.
    pub passwd: Option<SecretRef>,

    /// Whether TLS should be used for the database connection.
    pub ssl: Option<bool>,
}

impl DatabaseConfigBuilder {

    /// Sets the database name.
    pub fn database(mut self, name: impl Into<String>) -> Self {
        self.database = Some(name.into());
        self
    }

    /// Sets the database user.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Sets the password secret reference.
    pub fn password(mut self, passwd: SecretRef) -> Self {
        self.passwd = Some(passwd);
        self
    }

    /// Enables or disables TLS for the database connection.
    pub fn ssl(mut self, enabled: bool) -> Self {
        self.ssl = Some(enabled);
        self
    }

    /// Sets the database driver.
    pub fn driver(mut self, driver: DatabaseDriver) -> Self {
        self.driver = Some(driver);
        self
    }

    /// Validates and builds a [`DatabaseConfig`].
    pub fn build(self) -> Result<DatabaseConfig, DatabaseConfigError> {
        
        Ok(DatabaseConfig {
            host: self.host.ok_or(DatabaseConfigError::MissingHost)?,
            port: self.port.ok_or(DatabaseConfigError::MissingPort)?,
            database: self.database.ok_or(DatabaseConfigError::MissingDatabase)?,
            user: self.user.ok_or(DatabaseConfigError::MissingUser)?,
            passwd: self.passwd.ok_or(DatabaseConfigError::MissingPasswd)?,
            driver: self.driver.ok_or(DatabaseConfigError::MissingDriver)?,
            ssl: self.ssl,
        })
    }

    /// Returns whether all required database fields appear to be set.
    pub fn is_setup(&self) -> bool {
        self.host.is_some()
            && self.port.is_some()
            && self.database.is_some()
            && self.user.is_some()
            && self.passwd.is_some()
    }

    /// Returns the database socket address (`host:port`).
    pub fn socket_addr(&self) -> Result<String, DatabaseConfigError> {
        Ok(format!(
            "{}:{}",
            self.host.as_ref().ok_or(DatabaseConfigError::MissingHost)?,
            self.port.as_ref().ok_or(DatabaseConfigError::MissingPort)?
        ))
    }

    /// Returns whether the configured database host refers to a local address.
    pub fn is_local(&self) -> Result<bool, DatabaseConfigError> {
        Ok(matches!(
            self.host.as_ref().ok_or(DatabaseConfigError::MissingHost)?.as_str(),
            "localhost" | "127.0.0.1" | "::1"
        ))
    }
}