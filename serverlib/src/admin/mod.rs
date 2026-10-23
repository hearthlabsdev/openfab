pub mod apikeys;
pub mod groups;
pub mod permissions;
pub mod reports;
pub mod users;

use crate::accounts::User;
use ormlite::Model;
use ormlite::model::Join;
use ormlite::model::JoinMeta;
use ormlite::postgres::PgPool;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Model, Serialize)]
#[ormlite(table = "user_activation_code")]
pub struct ActivationCodeORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    /// the user who created this activation code.
    #[ormlite(column = "creator")]
    pub creator: Join<User>,
    /// the user who's account this activation code is for.
    #[ormlite(column = "account")]
    pub account: Join<User>,
    code: String,
    used: bool,
    invalidated: bool,
    invalidation_reason: Option<String>,
}

impl ActivationCodeORM {
    pub async fn get_code(pool: &PgPool, uid: Uuid) -> Result<ActivationCodeORM, ormlite::Error> {
        let mut conn = pool.acquire().await?;
        Ok(Self::select()
            .where_("uid = ?")
            .bind(uid)
            .fetch_one(&mut *conn)
            .await?)
    }
    pub async fn delete_code(pool: &PgPool, code: Uuid) -> Result<(), ormlite::Error> {
        let mut conn = pool.acquire().await?;
        let code = Self::get_code(pool, code).await?;
        code.delete(&mut *conn).await?;
        Ok(())
    }
}
