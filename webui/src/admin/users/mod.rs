//! This module is intended to handle listing active users
pub mod forms;
pub mod ui;

use crate::utils::unix_epoch_seconds;
use ormlite::Model;
use rand::{Rng, distr::Alphanumeric};
use serde_derive::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Represents a single activation code used to link a User → OIDC Subject.
#[derive(Debug, Clone, Serialize, Deserialize, Model)]
pub struct UserActivationCode {
    /// Primary key for this activation record
    #[ormlite(primary_key)]
    pub uid: Uuid,

    pub creator: Uuid,
    /// The user this activation code applies to
    pub account: Uuid,

    /// The actual activation code (short, random, single-use)
    pub code: String,

    /// Expiration timestamp
    /// in unix epoch so I don't have to deal with timezone BS
    pub expires_at: Option<i64>,
    pub created_at: i64,
    /// Whether the activation code has been redeemed
    pub used: bool,
    /// Whether the code has been force-invalidated by the system/admin
    pub invalidated: bool,

    /// Optional reason for invalidation
    pub invalidation_reason: Option<String>,
}

impl UserActivationCode {
    /// Generate a secure activation code for `user_uid`.
    pub fn generate_for_user(creator: Uuid, user_uid: Uuid, ttl_secs: Option<i64>) -> Self {
        let now = unix_epoch_seconds();

        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();

        Self {
            uid: Uuid::new_v4(),
            creator,
            account: user_uid,
            code,
            expires_at: ttl_secs.map(|v| now + v),
            used: false,
            invalidated: false,
            invalidation_reason: None,
            created_at: now,
        }
    }

    /// Check if the code is expired.
    pub fn is_expired(&self) -> bool {
        let now = unix_epoch_seconds();

        match self.expires_at {
            Some(v) => v <= now,
            None => false,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.invalidated
    }
}
