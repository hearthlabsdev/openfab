pub mod ui;
use ormlite::Model;
use rocket::FromForm;
use rocket::form::Form;
use rocket_oidc::claims::CoreClaims;
use rocket_oidc::utils::string_or_vec;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use time::{Duration, OffsetDateTime};
pub use ui::get_routes;
use uuid::Uuid;

use crate::utils;

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("passwords don't match")]
    PasswordMismatch,
    #[error("failed to hash password")]
    PasswordHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    allow_registration: bool,
}

impl AccountConfig {
    pub fn allow_registration(&self) -> bool {
        self.allow_registration
    }
}

impl Default for AccountConfig {
    fn default() -> AccountConfig {
        Self {
            allow_registration: true,
        }
    }
}

#[derive(Debug, Clone, FromForm)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, FromForm)]
pub struct RegisterForm {
    pub email: String,
    pub fname: String,
    pub lname: String,
    pub password: String,
    pub password2: String,
}

impl RegisterForm {
    pub fn register(self) -> Result<User, AccountError> {
        if self.password != self.password2 {
            return Err(AccountError::PasswordMismatch);
        }

        let password =
            utils::hash_password(&self.password).map_err(|e| AccountError::PasswordHash)?;

        Ok(User {
            uid: Uuid::new_v4(),
            subject: None,
            fname: self.fname,
            lname: self.lname,
            email: self.email,
            profile_image: None,
            password: Some(password),
            created: utils::unix_epoch_seconds(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "users")]
pub struct User {
    #[ormlite(primary_key)]
    uid: Uuid,
    subject: Option<String>,
    fname: String,
    lname: String,
    email: String,
    profile_image: Option<Uuid>,
    created: i64,
    password: Option<String>,
}

impl User {
    pub fn redact_pwd(mut self) -> Self {
        self.password = None;
        self
    }

    pub fn uid(&self) -> Uuid {
        self.uid
    }

    pub fn generate_local_user_id(&self, subject: Option<String>) -> UserID {
        let sub = match subject {
            Some(value) => value,
            None => self.email.clone(),
        };

        let hour = OffsetDateTime::now_utc()
            .checked_add(Duration::new(3600, 0))
            .expect("failed to add 1 hour")
            .unix_timestamp();

        let now = OffsetDateTime::now_utc().unix_timestamp();

        UserID {
            sub,
            iss: vec!["http://localhost".into()],
            iat: now,
            exp: hour,
            aud: vec!["self".into()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserID {
    pub sub: String,
    #[serde(deserialize_with = "string_or_vec")]
    iss: Vec<String>,
    iat: i64,
    exp: i64,
    #[serde(deserialize_with = "string_or_vec")]
    aud: Vec<String>,
}

impl CoreClaims for UserID {
    fn subject(&self) -> &str {
        &self.sub
    }
    fn issuer(&self) -> Vec<String> {
        self.iss.clone()
    }
    fn audience(&self) -> Vec<String> {
        self.aud.clone()
    }
    fn issued_at(&self) -> i64 {
        self.iat
    }
    fn exp(&self) -> i64 {
        self.exp
    }
}
