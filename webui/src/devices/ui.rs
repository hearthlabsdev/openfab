use crate::devices::PrinterDiscoveryEventORM;
use crate::devices::forms::DeviceUploadForm;
use crate::devices::{DeviceConfigORM, DeviceORM};
use crate::prints::{PrintJobORM, PrintQueueORM};
use crate::utils::{Guard, unix_epoch_seconds};

use openfab_drivers::native::NativeRuntime;
use openfab_drivers::runtime::DriverRuntime;
use openfab_drivers::utils::config_to_json;
use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::State;
use rocket::response::content::RawHtml;
use rocket::{Route, form::Form, get, post, response::Redirect, routes};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

use std::collections::HashMap;

#[get("/")]
pub async fn index(pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let devices = DeviceORM::select().fetch_all(&mut *conn).await.unwrap();

    let context = context! { devices };
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
    pool: &State<PgPool>,
    form: Form<HashMap<String, String>>,
) -> Redirect {
    let form = form.into_inner();
    #[cfg(debug_assertions)]
    println!("received form: {}", serde_json::to_string(&form).unwrap());
    let driver_id = form.get("driver").unwrap();
    let schema = rt.config_schema(&driver_id).await.unwrap();
    let value = config_to_json(&schema, &form).unwrap();
    rt.add_device(driver_id, value).await.unwrap();
    let name = form.get("name").unwrap().to_string();
    if name == "" {
        panic!("name not set");
    }
    let mut queue_opt: Option<Uuid> = form.get("queue").map(|v| v.parse().unwrap());

    let mut conn = pool.acquire().await.unwrap();

    if queue_opt.is_none() {
        let queue = PrintQueueORM::new(format!("{} Default Queue", name));
        let queue = queue.insert(&mut *conn).await.unwrap();
        queue_opt = Some(queue.uid);
    }
    let device = DeviceORM {
        uid: Uuid::new_v4(),
        queue: queue_opt,
        name,
        first_seen: unix_epoch_seconds(),
        last_seen: unix_epoch_seconds(),
        status: "active".to_string(),
        enabled: true,
        serial: form.get("serial").unwrap().to_string(),
        driver_id: driver_id.to_string(),
        driver_version: "0.1.0".to_string(),
    };

    let device = device.insert(&mut *conn).await.unwrap();
    for (key, value) in form.into_iter() {
        let config = DeviceConfigORM {
            uid: Uuid::new_v4(),
            device: device.uid,
            key,
            value,
        };
        config.insert(&mut *conn).await.unwrap();
    }
    // also need to impl store and retrive from database, but that can wait.
    Redirect::to("/devices")
}

#[get("/<uid>")]
pub async fn device(uid: Uuid, pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    // I need to query device, configs, get print queue, and lookup connected prints.
    let device = DeviceORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    RawHtml(Template::render(
        "pages/devices/overview",
        context! { device },
    ))
}

#[get("/<uid>/queue")]
pub async fn device_queue(uid: Uuid, pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    // I need to query device, configs, get print queue, and lookup connected prints.
    let device = DeviceORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    let queue = device.queue.unwrap();
    let print_jobs = PrintJobORM::select()
        .join(PrintJobORM::user())
        .join(PrintJobORM::asset())
        .where_("queue = ?")
        .bind(queue)
        .fetch_all(&mut *conn)
        .await
        .unwrap();

    RawHtml(Template::render(
        "pages/devices/queue",
        context! { device, queue: print_jobs },
    ))
}

#[get("/<uid>/configuration")]
pub async fn device_config(uid: Uuid, pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    // I need to query device, configs, get print queue, and lookup connected prints.
    let device = DeviceORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    RawHtml(Template::render(
        "pages/devices/configuration",
        context! { device },
    ))
}

#[get("/<uid>/captures")]
pub async fn device_captures(uid: Uuid, pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    // I need to query device, configs, get print queue, and lookup connected prints.
    let device = DeviceORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    RawHtml(Template::render(
        "pages/devices/captures",
        context! { device },
    ))
}

#[get("/<uid>/files")]
pub async fn device_files(uid: Uuid, pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    // I need to query device, configs, get print queue, and lookup connected prints.
    let device = DeviceORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    RawHtml(Template::render("pages/devices/files", context! { device }))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        index,
        create_page,
        create,
        device,
        device_queue,
        device_config,
        device_captures,
        device_files
    ]
}
