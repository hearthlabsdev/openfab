pub mod apikeys;
pub mod groups;
pub mod permissions;
pub mod reports;
pub mod users;

use crate::accounts::User;
use ormlite::Model;
use ormlite::model::Join;
use ormlite::model::JoinMeta;
use serde::{Deserialize, Serialize};
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

pub fn get_routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    routes.extend(users::ui::get_routes());
    routes.extend(groups::ui::get_routes());
    routes.extend(apikeys::get_routes());
    routes.extend(reports::get_routes());
    routes
}
