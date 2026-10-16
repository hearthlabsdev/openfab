use crate::errors::OpenFabErr;
use crate::settings::SettingsORM;
use crate::utils::unix_epoch_seconds;
use ormlite::{Model, postgres::PgPool};
use rocket::http::CookieJar;
use rocket::{Route, State, get, put, response::content::RawCss, routes};
use rocket_dyn_templates::{Template, context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "themes")]
pub struct ThemeORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,

    pub background: String,
    pub surface: String,
    pub surface_hover: String,
    pub surface_header: String,

    pub border: String,
    pub border_subtle: String,

    pub text: String,
    pub text_strong: String,
    pub text_muted: String,

    pub accent: String,
    pub created_at: i64,
}

impl Default for ThemeORM {
    fn default() -> Self {
        Self {
            uid: Uuid::new_v4(),
            name: "Default".to_string(),

            background: "#0f1117".to_string(),
            surface: "#12151d".to_string(),
            surface_hover: "#1a2030".to_string(),
            surface_header: "#151922".to_string(),

            border: "#232938".to_string(),
            border_subtle: "#2a3040".to_string(),

            text: "#e6e6e6".to_string(),
            text_strong: "#ffffff".to_string(),
            text_muted: "#7a8395".to_string(),

            accent: "#4c8dff".to_string(),
            created_at: unix_epoch_seconds(),
        }
    }
}

pub struct ThemePicker {}

impl ThemePicker {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn select(jar: &CookieJar<'_>, pool: &PgPool) -> Result<ThemeORM, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        let theme_id: Uuid = match jar.get("theme") {
            Some(value) => { println!("theme: {}", value.to_string()); value.to_string().parse().expect("failed to get theme cookie") },
            None => SettingsORM::select()
                .where_("key = 'theme'")
                .fetch_one(&mut *conn)
                .await
                .map(|v| v.value.parse())
                .unwrap().expect("failed to get theme"),//_or(Ok(Uuid::nil()))?,
        };
        println!("theme id: {}", theme_id);
        let theme = if theme_id == Uuid::nil() {
            ThemeORM::default()
        } else {
            ThemeORM::select()
                .where_("uid = ?")
                .bind(theme_id)
                .fetch_one(&mut *conn)
                .await.expect("failed to feme theme")
        };
        Ok(theme)
    }
}

// this provides the default / currently set server side base theme
#[get("/base.css")]
pub async fn base_theme(jar: &CookieJar<'_>, pool: &State<PgPool>) -> RawCss<Template> {
    
    let theme = ThemePicker::select(jar, pool).await.unwrap();
    println!("theme: {:?}", theme);
    RawCss(Template::render("styles/base", context! { theme }))
}

// this provides the default / currently set server side acount pages theme
#[get("/accounts.css")]
pub async fn accounts_theme(jar: &CookieJar<'_>, pool: &State<PgPool>) -> RawCss<Template> {
    let theme = ThemePicker::select(jar, pool).await.unwrap();
    RawCss(Template::render("styles/accounts", context! { theme }))
}

#[get("/<uid>/base.css")]
pub async fn get_named_base_theme(pool: &State<PgPool>, uid: Uuid) -> RawCss<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let theme = ThemeORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    RawCss(Template::render("styles/base", context! { theme }))
}

#[get("/<uid>/accounts.css")]
pub async fn get_named_accounts_theme(pool: &State<PgPool>, uid: Uuid) -> RawCss<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let theme = ThemeORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    RawCss(Template::render("styles/accounts", context! { theme }))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        base_theme,
        accounts_theme,
        get_named_accounts_theme,
        get_named_base_theme
    ]
}
