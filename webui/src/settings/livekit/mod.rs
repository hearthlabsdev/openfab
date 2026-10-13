/*use crate::errors::OpenFabErr;
use livekit::access_token::AccessToken;
use ormlite::Model;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "lk_server")]
pub struct LKServerORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub url: String,
    pub api_key: String,
    pub api_secret: String,
    /// when the server api_key, api_secret where added.
    pub added: i64,
}

impl LKServerORM {
    pub fn client_grant(&self) -> Result<String, OpenFabErr> {
        let token = AccessToken::with_api_key(&self.api_key, &self.api_secret);
        unimplemented!();
    }
}
*/