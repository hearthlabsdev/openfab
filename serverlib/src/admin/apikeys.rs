use crate::admin::User;
use crate::errors::AuthErr;
use crate::utils::{random_string, unix_epoch_seconds};
use ormlite::model::Join;
use ormlite::model::JoinMeta;
use ormlite::{Model, postgres::PgPool};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Model)]
#[ormlite(table = "api_keys")]
pub struct ApiKeyORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub keyid: String,
    #[ormlite(column = "owner")]
    pub owner: Join<User>,
    /// the user who's permissions this key is allowed to inherit
    /// this is different from the owner, which is the account that created the key
    /// this is used to assign a bot user to the key, so that it can be used to perform actions as a restricted user.
    #[ormlite(column = "account")]
    pub account: Join<User>,
    pub access_key: String,
    // expiration as unix epoch
    pub expires_at: i64,
    pub created_at: i64,
}

impl ApiKeyORM {
    pub async fn get_by_key(pool: &PgPool, key: &str) -> Result<Self, AuthErr> {
        let keyid = key.split(':').next().ok_or(AuthErr::ApiKeyNotFound)?;
        let access_key = key.split(':').nth(1).ok_or(AuthErr::ApiKeyNotFound)?;
        let api_key = Self::get_by(pool, keyid).await?;

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

    async fn get_by(pool: &PgPool, value: &str) -> Result<Self, AuthErr> {
        let mut conn = pool.acquire().await?;
        Self::select()
            .where_("keyid = ?")
            .bind(value)
            .fetch_one(&mut *conn)
            .await
            .map_err(|_| AuthErr::ApiKeyNotFound)
    }

    pub async fn generate_for_user(pool: &PgPool, user: &User) -> Result<Self, AuthErr> {
        let mut conn = pool.acquire().await?;
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
            .insert(&mut *conn)
            .await
            .map_err(|_| AuthErr::ApiKeyCreationFailed)?;
        Ok(api_key)
    }

    pub async fn list_for_owner(pool: &PgPool, owner_uid: Uuid) -> Result<Vec<Self>, AuthErr> {
        let mut conn = pool.acquire().await?;
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

    pub async fn delete_key(pool: &PgPool, key: Uuid) -> Result<(), ormlite::Error> {
        let mut conn = pool.acquire().await?;
        let key = Self::select()
            .where_("uid = ?")
            .bind(key)
            .fetch_one(&mut *conn)
            .await?;

        key.delete(&mut *conn).await?;
        Ok(())
    }
}
