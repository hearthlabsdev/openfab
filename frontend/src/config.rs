use protocol::config::{ServerConfig, DatabaseConfig};
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    database: DatabaseConfig,
    server: ServerConfig,
}