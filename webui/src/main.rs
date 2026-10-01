//! # Components
//! 1. DNS-SD advertises queues.
//! 2. IPP attributes describe capabilities.
//! 3. Device selection is an internal policy decision

use rocket::config::Config;
use rocket::fs::FileServer;
use rocket::response::content::RawHtml;
use rocket::data::{Limits, ToByteUnit};
use rocket::response::Redirect;
use rocket_dyn_templates::{Template, context};
use rocket_oidc::{sign::OidcSigner, config::OIDCConfig, auth::AuthState, client::LocalClient, config::WorkingConfig};
use std::net::{Ipv4Addr, SocketAddr};
use webui::VirtualPrinter;
use ormlite::postgres::PgPool;
use webui::library::ObjectStoreConfig;
use webui::ui::themes::ThemePicker;
use webui::accounts::AccountConfig;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> RawHtml<Template> {
    let context = context! {};
    RawHtml(Template::render("pages/index", &context))
}

#[catch(401)]
fn unauthorized() -> Redirect {
    Redirect::to("/accounts/login")
}

#[launch]
async fn rocket() -> _ {
    // Build Rocket
    let tx = VirtualPrinter::new().spawn_advertiser();

    println!("current time UTC: {}", time::OffsetDateTime::now_utc());

    let custom_limits = Limits::default()
        .limit("data-form", 100.megabytes())
        .limit("file", 100.megabytes());

    let config = Config {
        port: 7777,
        limits: custom_limits,
        ..Config::debug_default()
    };

    let store_config = ObjectStoreConfig {
        endpoint: "localhost:3900".to_string(),
        access_key: "GKaf44ea80663a6b5b5d15cb0d".to_string(),
        secret_key: "file:///home/cardinal/projects/hearthlabs/openfab/keys/store.txt".parse().expect("failed to create secret ref"),
        bucket: "openfab".to_string(),
        secure: false,
    };

    let oidc = OIDCConfig {
        name: "LaunchSpace (keycloak)".into(),
        client_id: "openfab".into(),
        client_secret: "file:///home/cardinal/projects/hearthlabs/openfab/keys/keycloak.txt".parse().expect("failed to create secret ref"),
        issuer_url: "http://localhost:3883/realms/master".into(),
        redirect: "http://localhost:7777".into(),
        privkey: None,
        post_login: Some("/accounts/dashboard".into()),
    };

    let store = store_config.object_store().await.unwrap();
    let accounts = AccountConfig::default();
    let pool = PgPool::connect("postgres://openfab:password@localhost:5432/openfab").await.unwrap();
    let rocket = rocket::custom(&config)
        .register("/", catchers![unauthorized])
        .mount("/", routes![index])
        .attach(Template::fairing())
        .manage(tx)
        .manage(accounts)
        .manage(ThemePicker::default())
        .manage(pool)
        .manage(vec![oidc.clone()])
        .manage(store)
        .mount("/static", FileServer::from("static"));

    let rocket = webui::ui::register_routes(rocket);
    let rocket = webui::api::register_routes(rocket);

    let mut key = File::open("../keys/backend.key.pem").await.unwrap();
    let mut priv_key_str = String::new();
    key.read_to_string(&mut priv_key_str).await.unwrap();

    let working_config = WorkingConfig::new_local("/").expect("failed to create working config");
    let signer = OidcSigner::from_rsa_pem(&priv_key_str, "0").expect("failed to create JWT signer");
    let client = LocalClient::new(working_config, signer).expect("failed to create local client");
    let state = AuthState::from_oidc_config(oidc).await.unwrap();
    state.set_local_client(client).await;
    state.setup(rocket)
}
