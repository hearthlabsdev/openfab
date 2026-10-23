
use serde::{Serialize, Deserialize};
use uuid::Uuid;
#[cfg(feature = "schema")]
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct LoginInfo {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ApiKeyInfo {
    pub uid: Uuid,
    pub keyid: String,
    pub owner: Uuid,
    pub account: Uuid,
    pub access_key: String,
    // expiration as unix epoch
    pub expires_at: i64,
    pub created_at: i64,
}

impl ApiKeyInfo {
    pub fn get_bearer(&self) -> String {
        format!("{}:{}", self.keyid, self.access_key)
    }
}