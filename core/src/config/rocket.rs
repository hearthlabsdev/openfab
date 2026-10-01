use rocket::config::{Config, TlsConfig};
use crate::config::ServerConfig;
use crate::config::ConfigError;
use crate::errors::ApiError;
use crate::config::WebServerEngine;
use crate::config::ServerConfigError;

use secret_ref::SecretPolicy;
use rocket::{Rocket, Build};

#[async_trait::async_trait]
impl WebServerEngine for Rocket<Build> {
    async fn setup_configuration(server: &ServerConfig, policy: SecretPolicy) -> Result<Self, ConfigError> {
        // ssl must be intentionally disabled
        let tls = if let Some(false) = server.ssl {
            None
        }else{
            let cert = server.cert_file.as_ref().ok_or(ConfigError::Server(ServerConfigError::MissingTlsMaterial))?.fetch(policy.clone()).await?;
            let key = server.key_file.as_ref().ok_or(ConfigError::Server(ServerConfigError::MissingTlsMaterial))?.fetch(policy).await?;
            Some(TlsConfig::from_bytes(cert.expose().as_bytes(), key.expose().as_bytes()))
        };
        let config = rocket::Config {
            address: server.addr,
            port: server.port,
            tls,
            ..rocket::Config::default()
        };
        Ok(rocket::custom(config))
    }
}