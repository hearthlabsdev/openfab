//! Core configuration types used by the crate.

pub mod builders;

//#[cfg(feature = "rocket")]
// Rocket-specific configuration integration.
//pub mod rocket;

use builders::*;
use crate::errors::*;

use std::path::PathBuf;
use std::net::IpAddr;

use secret_ref::SecretRef;
use secret_ref::{SecretPolicy};

use std::fmt;

/// Server configuration describing how the web service should run.
///
/// Note: In many deployments TLS certificate management is handled by
/// a reverse proxy such as Caddy or nginx rather than the application
/// itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Optional path to the configuration file used to load this config.
    pub config_file: Option<PathBuf>,

    /// Whether TLS/SSL is enabled.
    pub ssl: Option<bool>,

    /// The IP address the server should bind to.
    pub addr: IpAddr,

    /// Hostname used for reflection or external references.
    pub host: String,

    /// TCP port the server listens on.
    pub port: u16,

    /// Reference to the TLS certificate file.
    pub cert_file: Option<SecretRef>,

    /// Reference to the TLS private key file.
    pub key_file: Option<SecretRef>,

    /// Whether a self-signed certificate should be generated.
    pub self_signed: Option<bool>,
}

/// Trait implemented by web server backends capable of using
/// [`ServerConfig`] to initialize themselves.
#[async_trait::async_trait]
pub trait WebServerEngine: Sized {

    /// Performs engine-specific setup using the provided server configuration
    /// and secret resolution policy.
    async fn setup_configuration(
        server: &ServerConfig,
        policy: SecretPolicy,
    ) -> Result<Self, ConfigError>;
}

/// Represents an externally reachable API endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct APIEndpoint {
    /// Hostname or IP address of the endpoint.
    pub host: String,

    /// TCP port used by the endpoint.
    pub port: u16,

    /// Whether TLS/SSL is enabled for the endpoint.
    pub ssl: Option<bool>,
}

impl APIEndpoint {

    /// Creates a new API endpoint.
    ///
    /// Returns an error if the provided port is zero.
    pub fn new(host: impl Into<String>, port: u16) -> Result<Self, ApiEndpointError> {
        if port == 0 {
            return Err(ApiEndpointError::InvalidPort);
        }

        Ok(Self {
            host: host.into(),
            port,
            ssl: Some(false),
        })
    }

    /// Returns whether the endpoint has the minimal required fields set.
    pub fn is_setup(&self) -> bool {
        !self.host.is_empty()
    }

    /// Enables or disables TLS for the endpoint.
    pub fn with_ssl(mut self, enabled: bool) -> Self {
        self.ssl = Some(enabled);
        self
    }

    /// Returns the host component.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the port component.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns whether TLS is enabled.
    pub fn ssl(&self) -> bool {
        self.ssl.unwrap_or(false)
    }

    /// Returns the URL scheme (`http` or `https`).
    pub fn scheme(&self) -> &'static str {
        if self.ssl != Some(false) { "https" } else { "http" }
    }

    /// Returns the `host:port` authority string.
    pub fn authority(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Returns the full base URL of the endpoint.
    pub fn url(&self) -> String {
        format!("{}://{}", self.scheme(), self.authority())
    }
}

/// Supported database driver types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DatabaseDriver {
    /// PostgreSQL database driver.
    Postgres,

    /// MySQL database driver.
    MySql,

    /// MariaDB database driver.
    MariaDb,

    /// SQLite database driver.
    Sqlite,

    /// Microsoft SQL Server driver.
    #[serde(rename = "sqlserver", alias = "mssql")]
    MicrosoftSql,

    /// Oracle database driver.
    Oracle,

    /// Custom or unsupported driver name.
    #[serde(untagged)]
    Other(String),
}

impl fmt::Display for DatabaseDriver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            Self::Postgres => "postgres",
            Self::MySql => "mysql",
            Self::MariaDb => "mariadb",
            Self::Sqlite => "sqlite",
            Self::MicrosoftSql => "sqlserver",
            Self::Oracle => "oracle",
            Self::Other(val) => val.as_str(),
        };
        write!(f, "{}", val)
    }
}

/// Database connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database driver implementation.
    pub driver: DatabaseDriver,

    /// Database host address.
    pub host: String,

    /// Database port.
    pub port: u16,

    /// Database name.
    pub database: String,

    /// Database username.
    pub user: String,

    /// Secret reference to the database password.
    pub passwd: SecretRef,

    /// Whether TLS should be used for the database connection.
    pub ssl: Option<bool>,
}

impl DatabaseConfig {

    /// Creates a [`DatabaseConfigBuilder`] initialized with a host and port.
    ///
    /// Returns an error if the port is zero.
    pub fn builder(
        host: impl Into<String>,
        port: u16,
    ) -> Result<DatabaseConfigBuilder, DatabaseConfigError> {
        if port == 0 {
            return Err(DatabaseConfigError::InvalidPort);
        }

        Ok(DatabaseConfigBuilder {
            host: Some(host.into()),
            port: Some(port),
            driver: None,
            database: None,
            user: None,
            passwd: None,
            ssl: None,
        })
    }

    /// Returns whether TLS is enabled for the database connection.
    pub fn ssl_enabled(&self) -> bool {
        self.ssl.unwrap_or(false)
    }

    /// Builds a database connection URL, resolving the password using
    /// the provided [`SecretPolicy`].
    pub async fn url(&self, policy: SecretPolicy) -> Result<String, ConfigError> {
        Ok(format!(
            "{}://{}:{}@{}/{}",
            self.driver,
            self.user,
            self.passwd.fetch(policy).await?.expose(),
            self.host,
            self.database
        ))
    }
}