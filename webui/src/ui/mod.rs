pub mod dashboard;
pub mod prints;
pub mod themes;
use crate::admin;
use rocket::{Build, Rocket};

pub fn register_routes(r: Rocket<Build>) -> Rocket<Build> {
    r.mount("/library", crate::library::get_routes())
        .mount("/devices", crate::devices::get_routes())
        .mount("/prints", prints::get_routes())
        .mount("/accounts", crate::accounts::get_routes())
        .mount("/admin", crate::admin::get_routes())
        .mount("/css", themes::get_routes())
}
