//! This module is intended to handle listing active users

use crate::utils::random_string;
use crate::utils::unix_epoch_seconds;
use ormlite::Model;
use crate::admin::groups::GroupORM;
use crate::accounts::User;
use ormlite::postgres::PgPool;
use crate::errors::OpenFabErr;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub uid: Uuid,
    pub subject: Option<String>,
    pub fname: String,
    pub lname: String,
    pub email: String,
    pub profile_image: Option<Uuid>,
    pub created: i64,
    pub last_login: i64,
    pub groups: Vec<GroupORM>,   
}

impl UserInfo {
    pub async fn list_paginated(pool: &PgPool, limit: usize, offset: usize) -> Result<Vec<UserInfo>, OpenFabErr> {
        let mut conn = pool.acquire().await?;

        // 1. Fetch the users page
        let users = User::select()
            .limit(limit)
            .offset(offset)
            .fetch_all(&mut *conn)
            .await?;

        if users.is_empty() {
            return Ok(Vec::new());
        }

        // 2. Extract user Uuids
        let user_ids: Vec<Uuid> = users.iter().map(|u| u.uid).collect();

        // 3. Fetch all groups for ALL these users in ONE query
        // We join group_membership with groups, filtering where the account is in our user list
        let query = r#"
            SELECT 
                gm.account as user_uid,
                g.uid, g.name, g.description
            FROM group_membership gm
            JOIN groups g ON gm.account_group = g.uid
            WHERE gm.account = ANY($1)
        "#;

        // A helper struct to capture the intermediate SQL result
        #[derive(sqlx::FromRow)]
        struct UserGroupRow {
            user_uid: Uuid,
            uid: Uuid,
            name: String,
            description: Option<String>,
        }

        let group_rows: Vec<UserGroupRow> = sqlx::query_as(query)
            .bind(&user_ids)
            .fetch_all(&mut *conn)
            .await?;

        // 4. Group the groups by user_uid using a HashMap
        let mut user_groups_map: HashMap<Uuid, Vec<GroupORM>> = HashMap::new();
        for row in group_rows {
            let group = GroupORM {
                uid: row.uid,
                name: row.name,
                description: row.description,
            };
            user_groups_map.entry(row.user_uid).or_default().push(group);
        }

        // 5. Combine users and their groups into UserInfo structs
        let user_infos = users.into_iter().map(|user| {
            let groups = user_groups_map.remove(&user.uid).unwrap_or_default();
            UserInfo {
                uid: user.uid,
                subject: user.subject,
                fname: user.fname,
                lname: user.lname,
                email: user.email,
                profile_image: user.profile_image,
                created: user.created,
                last_login: user.last_login,
                groups,
            }
        }).collect();

        Ok(user_infos)
    }
}

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

        let code = random_string(16);

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
