use ormlite::Model;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "api_keys")]
pub struct ApiKeyORM {
    #[ormlite(primary_key)]
    uid: Uuid,
    keyid: String,
    owner: Uuid,
    /// the user who's permissions this key is allowed to inherit
    account: Option<Uuid>,
    access_key: String,
    // expiration as unix epoch
    expires: i64,
    created: i64,
}

/*
impl ApiKeyORM {
    pub fn generate(owner: Uuid, account: Uuid, expires: i64, keyid: Option<String>) -> Self {
        unimplemented!();
    }
}*/
