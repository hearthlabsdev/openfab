pub mod forms;
pub mod ui;

use crate::accounts::User;
use ormlite::Model;
use ormlite::model::{Join, JoinMeta};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "groups")]
pub struct GroupORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Model)]
#[ormlite(table = "group_membership")]
pub struct GroupMembershipRel {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    #[ormlite(column = "user")]
    pub user: Join<User>,
    #[ormlite(column = "group")]
    pub group: Join<GroupORM>,
}
