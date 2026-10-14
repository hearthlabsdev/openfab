use crate::admin::users::UserActivationCode;
use rocket::{form::FromForm, fs::TempFile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, FromForm)]
pub struct ActivationForm {
    pub subject: String,
    pub code: String,
}

#[derive(Debug, FromForm)]
pub struct UserForm<'r> {
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
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub picture: Option<TempFile<'r>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct ActivationCodeForm {
    pub user: Uuid,
    pub ttl: Option<i64>,
}
