pub mod admin;
pub mod dashboard;
pub mod devices;
pub mod prints;
pub mod themes;
use rocket::{Build, Rocket};

pub fn register_routes(r: Rocket<Build>) -> Rocket<Build> {
    r
        .mount("/library", crate::library::get_routes())
        .mount("/devices", devices::get_routes())
        .mount("/prints", prints::get_routes())
        .mount("/accounts", crate::accounts::get_routes())
        .mount("/admin", admin::get_routes())
        .mount("/css", themes::get_routes())
}