use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use rocket::{get, put, Route, routes, State, response::content::RawCss};
use serde::{Serialize, Deserialize};
use std::default::Default;
use rocket::http::{CookieJar};

pub struct ThemePicker {
    current: Arc<RwLock<String>>,
    themes: HashMap<String, Theme>,
}

impl Default for ThemePicker {
    fn default() -> ThemePicker {
        let mut map = HashMap::new();
        map.insert("default".to_string(), Theme::default());
        map.insert("dark".to_string(), Theme::dark());
        map.insert("light".to_string(), Theme::light());
        ThemePicker {
            current: Arc::new(RwLock::new("default".to_string())),
            themes: map,
        }
    }
}

impl ThemePicker {
    pub async fn select(&self) -> Theme {
        let current: String = self.current.read().await.to_string();
        match self.themes.get(&current) {
            Some(current) => current.clone(),
            None => Theme::default(),
        }
    }
    pub async fn by_name(&self, name: &str) -> Theme {
        match self.themes.get(name) {
            Some(theme) => theme.clone(),
            None => self.select().await
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct Theme {
    name: &'static str,
    base: &'static str,
    accounts: &'static str,
}

impl Theme {
    pub fn light() -> Self {
        Theme {
            name: "default",
            base: include_str!("../../css/light/base.css"),
            accounts: include_str!("../../css/light/accounts.css")
        }
    }
    pub fn dark() -> Self {
        Theme {
            name: "default",
            base: include_str!("../../css/dark/base.css"),
            accounts: include_str!("../../css/dark/accounts.css")
        }
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::dark()
    }
}

// this provides the default / currently set server side base theme
#[get("/base.css")]
pub async fn base_theme(jar: &CookieJar<'_>, themes: &State<ThemePicker>) -> RawCss<String> {
    RawCss(match jar.get("theme") {
        Some(value) => {
            themes.by_name(value.value()).await.base.to_string()
        },
        None => themes.select().await.base.to_string()
    })
    
}

// this provides the default / currently set server side acount pages theme
#[get("/accounts.css")]
pub async fn accounts_theme(jar: &CookieJar<'_>, themes: &State<ThemePicker>) -> RawCss<String> {
    RawCss(themes.select().await.accounts.to_string())
}

#[get("/<name>/base.css")]
pub async fn get_named_base_theme(themes: &State<ThemePicker>, name: String) -> RawCss<String> {
    RawCss(themes.by_name(&name).await.base.to_string())
}

#[get("/<name>/accounts.css")]
pub async fn get_named_accounts_theme(themes: &State<ThemePicker>, name: String) -> RawCss<String> {
    RawCss(themes.by_name(&name).await.accounts.to_string())
}

pub fn get_routes() -> Vec<Route> {
    routes![
        base_theme,
        accounts_theme,
        get_named_accounts_theme,
        get_named_base_theme
    ]
}