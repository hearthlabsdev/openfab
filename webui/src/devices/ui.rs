use crate::devices::PrinterDiscoveryEventORM;
use crate::devices::forms::DeviceUploadForm;
use crate::utils::Guard;
use openfab_drivers::native::NativeRuntime;
use openfab_drivers::runtime::DriverRuntime;
use openfab_drivers::utils::config_to_json;
use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::State;
use rocket::response::content::RawHtml;
use rocket::{Route, form::Form, get, post, response::Redirect, routes};
use rocket_dyn_templates::{Template, context};

use std::collections::HashMap;

#[get("/")]
pub fn index() -> RawHtml<Template> {
    let context = context! {};
    RawHtml(Template::render("pages/devices/index", &context))
}

#[get("/create")]
async fn create_page(
    guard: Guard,
    rt: &State<NativeRuntime>,
    pool: &State<PgPool>,
) -> RawHtml<Template> {
    // within this current model one of two things needs to be the case:
    // either the server is running on a lan where it can discover devices, or a client needs to discover relaying the discovery to the server.
    let mut conn = pool
        .acquire()
        .await
        .expect("failed to acquire db connection");
    let devices = PrinterDiscoveryEventORM::select()
        .fetch_all(&mut *conn)
        .await
        .expect("failed to fetch discovery events");
    let drivers = rt.index_drivers().await.unwrap();
    println!("drivers len: {}", drivers.len());
    let context = context! { devices, drivers };

    RawHtml(Template::render("pages/devices/create", &context))
}

#[post("/create", data = "<form>")]
async fn create(
    guard: Guard,
    rt: &State<NativeRuntime>,
    form: Form<HashMap<String, String>>,
) -> Redirect {
    let form = form.into_inner();
    #[cfg(debug_assertions)]
    println!("received form: {}", serde_json::to_string(&form).unwrap());
    let driver_id = form.get("driver").unwrap();
    let schema = rt.config_schema(&driver_id).await.unwrap();
    let value = config_to_json(&schema, &form).unwrap();
    rt.add_device(driver_id, value).await.unwrap();

    // also need to impl store and retrive from database, but that can wait.
    Redirect::to("/devices")
}

pub fn get_routes() -> Vec<Route> {
    routes![index, create_page, create]
}
