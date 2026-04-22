//! This module is intended to handle listing active users
pub mod routes;
pub mod forms;

use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::{distr::Alphanumeric, Rng};
use ormlite::Model;

/// defines a user instance for project/loan checkouts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserORM {
    /// the unique ID of this user
    pub uid: Uuid,
    /// OIDC subject field to map to OIDC provider info.
    pub subject: Option<String>,
    /// user first name
    pub firstname: String,
    /// user last name
    pub lastname: String,
    /// user email address
    pub email: String,
    /// phone number of the user
    pub phone: Option<String>,
    /// Are they staff or a regular member?
    pub staff: Option<bool>,
    
    /// an identifier for a stripe customer
    pub stripe_customer_id: Option<String>,
    /// the timezone of the user
    pub timezone: Option<String>,
    /// the regionalization for the user eg en-US
    pub locale: Option<String>,
    /// optional url to the user's profile picture
    pub picture: Option<String>,
}

/// Represents a single activation code used to link a User → OIDC Subject.
#[derive(Debug, Clone, Serialize, Deserialize, Model)]
pub struct UserActivationCode {
    /// Primary key for this activation record
    #[ormlite(primary_key)]
    pub uid: Uuid,

    /// The user this activation code applies to
    pub user_uid: Uuid,

    /// The actual activation code (short, random, single-use)
    pub code: String,

    /// Expiration timestamp
    /// in unix epoch so I don't have to deal with timezone BS
    pub expires_at: i64,

    /// Whether the activation code has been redeemed
    pub used: bool,
    /// Whether the code has been force-invalidated by the system/admin
    pub invalidated: bool,

    /// Optional reason for invalidation
    pub invalidation_reason: Option<String>,
}

impl UserActivationCode {
    /// Generate a secure activation code for `user_uid`.
    pub fn generate_for_user(user_uid: Uuid, ttl_secs: i64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let expires_at = now + ttl_secs;

        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();

        Self {
            uid: Uuid::new_v4(),
            user_uid,
            code,
            expires_at,
            used: false,
            invalidated: false,
            invalidation_reason: None,
        }
    }

    /// Check if the code is expired.
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        now >= self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.invalidated
    }
}