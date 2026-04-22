//! # Components
//! 1. DNS-SD advertises queues.
//! 2. IPP attributes describe capabilities.
//! 3. Device selection is an internal policy decision

use rocket::config::Config;
use rocket::fs::FileServer;
use rocket::response::content::RawHtml;
use rocket_dyn_templates::{Template, context};
use std::net::{Ipv4Addr, SocketAddr};
use webui::VirtualPrinter;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> RawHtml<Template> {
    let context = context! {};
    RawHtml(Template::render("pages/index", &context))
}

#[launch]
async fn rocket() -> _ {
    // Build Rocket
    let tx = VirtualPrinter::new().spawn_advertiser();

    let config = Config {
        port: 7777,
        temp_dir: "/tmp/config-example".into(),
        ..Config::debug_default()
    };

    rocket::custom(&config)
        .attach(Template::fairing())
        .manage(tx)
        .mount("/", routes![index])
        .mount("/library", webui::library::routes::get_routes())
        .mount("/devices", webui::devices::routes::get_routes())
        .mount("/prints", webui::prints::routes::get_routes())
        .mount("/accounts", webui::accounts::routes::get_routes())
        .mount("/admin", webui::admin::get_routes())
        .mount("/static", FileServer::from("static"))
}
