use openfab::config::{DatabaseConfig, ServerConfig};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    database: DatabaseConfig,
    server: ServerConfig,
}
