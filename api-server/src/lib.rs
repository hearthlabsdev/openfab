#![allow(unused)]
pub mod devices;
pub mod errors;
pub mod library;
pub mod prints;
pub mod utils;

use crate::errors::ErrorResponse;
use ormlite::postgres::PgPool;
use ormlite::{Model, model::ModelBuilder};
use serverlib::{admin::apikeys::ApiKeyORM};
use serverlib::errors::{OpenFabErr, AuthErr};
use serverlib::utils::unix_epoch_seconds;
use serverlib::accounts::User;
use okapi::merge::marge_spec_list as merge_spec_list;
use rocket::{Build, State, Rocket};
use rocket::{get, post, serde::json::Json};
use rocket_okapi::{
    openapi_get_routes,
    rapidoc::{GeneralConfig, HideShowConfig, RapiDocConfig, make_rapidoc},
    settings::UrlObject,
    swagger_ui::{SwaggerUIConfig, make_swagger_ui},
};
use openfab_sdk::models::{LoginInfo, ApiKeyInfo};

#[get("/openapi-spec.json")]
pub fn spec_route() -> &'static str {
    include_str!("openapi-spec.json")
}

#[post("/login", data = "<login>")]
pub async fn api_login(pool: &State<PgPool>, login: Json<LoginInfo>) -> Result<Json<ApiKeyInfo>, ErrorResponse> {
    let mut conn = pool.acquire().await.unwrap();
    let user = User::select()
        .where_("email = ?")
        .bind(&login.email)
        .fetch_one(&mut *conn)
        .await
        .map_err(|v| OpenFabErr::AuthErr(AuthErr::UserNotFound))?;

    if !serverlib::utils::verify_password(&login.password, &user.password.as_ref().unwrap()).unwrap() {
        return Err(AuthErr::InvalidPassword.into());
    }

    let key = ApiKeyORM::generate_for_user(pool, &user).await?;
    let user = user
        .update_partial()
        .last_login(unix_epoch_seconds())
        .update(&mut *conn)
        .await?;

    let info = ApiKeyInfo {
        uid: key.uid,
        keyid: key.keyid,
        owner: key.owner.uid,
        account: key.account.uid,
        access_key: key.access_key,
        expires_at: key.expires_at,
        created_at: key.created_at,
    };
    Ok(Json(info))
}

pub fn register_routes(r: Rocket<Build>) -> Rocket<Build> {
    let (library_routes, library_spec) = library::get_routes();
    let (device_routes, device_spec) = devices::get_routes();
    let (print_routes, print_spec) = prints::get_routes();

    let spec = merge_spec_list(&[
        ("/api/library", library_spec),
        ("/api/devices", device_spec),
        ("/api/prints", print_spec),
    ])
    .expect("failed to merge spec");
    let json = serde_json::to_string_pretty(&spec).unwrap();
    std::fs::write("openapi-spec.json", json).unwrap();
    r.mount(
        "/swagger-ui/",
        make_swagger_ui(&SwaggerUIConfig {
            urls: vec![UrlObject::new("OpenFab", "/api/openapi-spec.json")],
            ..Default::default()
        }),
    )
    .mount(
        "/rapidoc/",
        make_rapidoc(&RapiDocConfig {
            general: GeneralConfig {
                spec_urls: vec![UrlObject::new("OpenFab", "/api/openapi-spec.json")],
                ..Default::default()
            },
            hide_show: HideShowConfig {
                allow_spec_url_load: false,
                allow_spec_file_load: false,
                ..Default::default()
            },
            ..Default::default()
        }),
    )
    .mount("/api/library/", library_routes)
    .mount("/api/devices", device_routes)
    .mount("/api/prints", print_routes)
    .mount(
        "/api/",
        rocket::routes![
            spec_route,
            api_login,
            // print methods
        ],
    )
}
