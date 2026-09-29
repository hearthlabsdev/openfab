use openfab::oidc::OidcClaims;
use rocket_oidc::{auth::{ApiKeyGuard}, OIDCGuard};
use rocket_dyn_templates::tera::Context;


pub type ApiGuard = ApiKeyGuard<OidcClaims>;
pub type Guard = OIDCGuard<crate::accounts::UserID>;

pub fn global_context() -> Context {
    Context::new()
}

use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
    Argon2
};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();

    let hash = argon2.hash_password(password.as_bytes())?;

    Ok(hash.to_string())
}

pub fn verify_password(
    password: &str,
    hash: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn unix_epoch_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_secs() as i64
}
