//! This module is intended to handle listing active users
pub mod ui;
pub mod forms;

use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::{distr::Alphanumeric, Rng};
use ormlite::Model;

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