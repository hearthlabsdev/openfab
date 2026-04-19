//! # Components
//! 1. DNS-SD advertises queues.
//! 2. IPP attributes describe capabilities.
//! 3. Device selection is an internal policy decision

use webui::VirtualPrinter;
use rocket_dyn_templates::{Template, context};
use rocket::config::{Config};
use std::net::{Ipv4Addr, SocketAddr};

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> Template {
    let context = context! {};
    Template::render("pages/index", &context)
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

    rocket::custom(&config).attach(Template::fairing()).manage(tx).mount("/", routes![index])
}
