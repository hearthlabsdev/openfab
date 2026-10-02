use crate::accounts::AccountConfig;
use crate::library::ObjectStoreConfig;
use openfab::config::{DatabaseConfig, ServerConfig};
use rocket_oidc::config::OIDCConfig;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    database: DatabaseConfig,
    server: ServerConfig,
    oidc: Vec<OIDCConfig>,
    library: ObjectStoreConfig,
    accounts: AccountConfig,
}
