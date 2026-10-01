use rocket_dyn_templates::{context, Template};
use rocket::response::content::RawHtml;
use rocket::{get, routes, Route};
use ormlite::postgres::PgPool;
use rocket::State;
use ormlite::Model;
use crate::devices::PrinterDiscoveryEventORM;

#[get("/")]
pub fn index() -> RawHtml<Template> {
    let context = context! {};
    RawHtml(Template::render("pages/devices/index", &context))
}

#[get("/create")]
async fn create(pool: &State<PgPool>) -> RawHtml<Template> {
    
    // within this current model one of two things needs to be the case:
    // either the server is running on a lan where it can discover devices, or a client needs to discover relaying the discovery to the server.
    let mut conn = pool.acquire().await.expect("failed to acquire db connection");
    let devices = PrinterDiscoveryEventORM::select()
        .fetch_all(&mut *conn).await.expect("failed to fetch discovery events");
    let context = context! { devices };
    RawHtml(Template::render("pages/devices/create", &context))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        crate::devices::ui::index,
        crate::devices::ui::create,
    ]
}