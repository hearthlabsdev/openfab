use openfab::config::{DatabaseConfig, ServerConfig};
use serde_derive::{Deserialize, Serialize};
use rocket_oidc::config::OIDCConfig;
use crate::library::ObjectStoreConfig;
use crate::accounts::AccountConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    database: DatabaseConfig,
    server: ServerConfig,
    oidc: Vec<OIDCConfig>,
    library: ObjectStoreConfig,
    accounts: AccountConfig,
}
