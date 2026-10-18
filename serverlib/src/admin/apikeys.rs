use crate::admin::User;
use crate::errors::AuthErr;
use crate::utils::{random_string, unix_epoch_seconds};
use ormlite::Model;
use ormlite::model::Join;
use ormlite::model::JoinMeta;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Model)]
#[ormlite(table = "api_keys")]
pub struct ApiKeyORM {
    #[ormlite(primary_key)]
    uid: Uuid,
    keyid: String,
    #[ormlite(column = "owner")]
    owner: Join<User>,
    /// the user who's permissions this key is allowed to inherit
    /// this is different from the owner, which is the account that created the key
    /// this is used to assign a bot user to the key, so that it can be used to perform actions as a restricted user.
    #[ormlite(column = "account")]
    account: Join<User>,
    access_key: String,
    // expiration as unix epoch
    expires_at: i64,
    created_at: i64,
}

impl ApiKeyORM {
    pub async fn get_by_key(
        conn: &mut ormlite::postgres::PgConnection,
        key: &str,
    ) -> Result<Self, AuthErr> {
        let keyid = key.split(':').next().ok_or(AuthErr::ApiKeyNotFound)?;
        let access_key = key.split(':').nth(1).ok_or(AuthErr::ApiKeyNotFound)?;
        let api_key = Self::get_by(conn, keyid).await?;

        if api_key.access_key != access_key {
            #[cfg(debug_assertions)]
            eprintln!("API key mismatch: {} != {}", api_key.access_key, key);
            return Err(AuthErr::ApiKeyInvalid);
        }
        if api_key.expires_at < unix_epoch_seconds() {
            return Err(AuthErr::ApiKeyExpired);
        }
        Ok(api_key)
    }

    async fn get_by(
        conn: &mut ormlite::postgres::PgConnection,
        value: &str,
    ) -> Result<Self, AuthErr> {
        Self::select()
            .where_("keyid = ?")
            .bind(value)
            .fetch_one(conn)
            .await
            .map_err(|_| AuthErr::ApiKeyNotFound)
    }

    pub async fn generate_for_user(
        conn: &mut ormlite::postgres::PgConnection,
        user: &User,
    ) -> Result<Self, AuthErr> {
        let uid = Uuid::new_v4();
        let keyid = random_string(12);
        let access_key = random_string(32);
        let expires_at = unix_epoch_seconds() + 60 * 60 * 24 * 30; // 30 days
        let created_at = unix_epoch_seconds();
        let api_key = Self {
            uid,
            keyid,
            owner: Join::new(user.clone()),
            account: Join::new(user.clone()),
            access_key,
            expires_at,
            created_at,
        };
        let api_key = api_key
            .insert(conn)
            .await
            .map_err(|_| AuthErr::ApiKeyCreationFailed)?;
        Ok(api_key)
    }

    pub async fn list_for_owner(
        conn: &mut ormlite::postgres::PgConnection,
        owner_uid: Uuid,
    ) -> Result<Vec<Self>, AuthErr> {
        let api_keys = Self::select()
            .join(ApiKeyORM::owner())
            .join(ApiKeyORM::account())
            .where_("owner = ?")
            .bind(owner_uid)
            .fetch_all(&mut *conn)
            .await
            .map_err(|_| AuthErr::ApiKeyNotFound)?;
        Ok(api_keys)
    }
}
