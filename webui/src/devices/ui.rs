use crate::devices::PrinterDiscoveryEventORM;
use crate::devices::forms::DeviceUploadForm;
use crate::errors::OpenFabErr;
use crate::utils::{Guard, unix_epoch_seconds};
use openfab_drivers::events::Event;
use openfab_drivers::native::NativeRuntime;
use openfab_drivers::runtime::DriverRuntime;
use openfab_drivers::utils::config_to_json;
use ormlite::Model;
use ormlite::model::ModelBuilder;
use ormlite::postgres::PgPool;
use rocket::State;
use rocket::form::FromForm;
use rocket::futures::SinkExt;
use rocket::futures::StreamExt;
use rocket::futures::future::FutureExt;
use rocket::response::content::RawHtml;
use rocket::{Route, form::Form, get, post, response::Redirect, routes};
use rocket_dyn_templates::{Template, context};
use rocket_ws::{Channel, Stream, WebSocket};
use serverlib::devices::DeviceFileORM;
use serverlib::devices::DeviceLogORM;
use serverlib::devices::{DeviceConfigORM, DeviceORM};
use serverlib::library::{AssetORM, LibraryORM, ObjectStore};
use serverlib::prints::{PrintJobORM, PrintQueueORM};
use serverlib::setup::NativeManager;
use std::path::PathBuf;
use std::sync::Arc;
use std::task::Poll;
use tokio::sync::Mutex;
use tokio::sync::broadcast::{Receiver, Sender};
use uuid::Uuid;

use std::collections::HashMap;

#[get("/")]
pub async fn index(guard: Guard, pool: &State<PgPool>) -> Result<RawHtml<Template>, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let devices = DeviceORM::select().fetch_all(&mut *conn).await?;

    let context = context! { devices };
    Ok(RawHtml(Template::render("pages/devices/index", &context)))
}

#[get("/create")]
async fn create_page(
    guard: Guard,
    rt: &State<NativeManager>,
    pool: &State<PgPool>,
) -> Result<RawHtml<Template>, OpenFabErr> {
    // within this current model one of two things needs to be the case:
    // either the server is running on a lan where it can discover devices, or a client needs to discover relaying the discovery to the server.
    let mut conn = pool.acquire().await?;
    let devices = PrinterDiscoveryEventORM::select()
        .fetch_all(&mut *conn)
        .await?;
    let drivers = rt.index_drivers().await?;
    println!("drivers len: {}", drivers.len());
    let context = context! { devices, drivers };

    Ok(RawHtml(Template::render("pages/devices/create", &context)))
}

#[post("/create", data = "<form>")]
async fn create(
    guard: Guard,
    rt: &State<NativeManager>,
    pool: &State<PgPool>,
    form: Form<HashMap<String, String>>,
) -> Result<Redirect, OpenFabErr> {
    let mut form = form.into_inner();
    form.retain(|_, v| !v.is_empty());

    #[cfg(debug_assertions)]
    println!("received form: {}", serde_json::to_string(&form)?);
    let driver_id = form.get("driver").ok_or(OpenFabErr::MissingConfigOption(
        "devices/create".into(),
        "driver".into(),
    ))?;
    let schema = rt.config_schema(&driver_id).await?;

    let value = config_to_json(&schema, &form)?;
    rt.add_device(driver_id, value).await?;
    let name = form
        .get("name")
        .ok_or(OpenFabErr::MissingConfigOption(
            "devices/create".into(),
            "name".into(),
        ))?
        .to_string();
    if name.is_empty() {
        return Err(OpenFabErr::MissingConfigOption(
            "devices/create".into(),
            "name".into(),
        ));
    }
    let mut queue_opt: Option<Uuid> = form.get("queue").map(|v| v.parse().unwrap());

    let mut conn = pool.acquire().await?;

    if queue_opt.is_none() {
        let queue = PrintQueueORM::new(format!("{} Default Queue", name));
        let queue = queue.insert(&mut *conn).await?;
        queue_opt = Some(queue.uid);
    }
    let device = DeviceORM {
        uid: Uuid::new_v4(),
        queue: queue_opt,
        library: None,
        name,
        first_seen: unix_epoch_seconds(),
        last_seen: unix_epoch_seconds(),
        status: "active".to_string(),
        enabled: true,
        upload_dir: None,
        serial: form
            .get("serial")
            .ok_or(OpenFabErr::MissingConfigOption(
                "device.create".into(),
                "serial".into(),
            ))?
            .to_string(),
        driver_id: driver_id.to_string(),
        driver_version: "0.1.0".to_string(),
    };

    let device = device.insert(&mut *conn).await?;
    for (key, value) in form.into_iter() {
        let config = DeviceConfigORM {
            uid: Uuid::new_v4(),
            device: device.uid,
            key,
            value,
            updated_at: unix_epoch_seconds(),
        };
        config.insert(&mut *conn).await?;
    }
    // also need to impl store and retrive from database, but that can wait.
    Ok(Redirect::to("/"))
}

#[get("/<uid>/info")]
pub async fn device(guard: Guard, uid: Uuid, pool: &State<PgPool>) -> RawHtml<Template> {
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
pub async fn device_queue(guard: Guard, uid: Uuid, pool: &State<PgPool>) -> RawHtml<Template> {
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

#[derive(Debug, Clone, FromForm)]
pub struct ConfigUpdateForm {
    pub name: String,
    pub queue: Uuid,
    pub driver_id: String,
    pub serial: String,
    configs: HashMap<Uuid, String>,
}

#[post("/<uid>/configuration/edit", data = "<form>")]
pub async fn edit_device_config(
    guard: Guard,
    rt: &State<NativeManager>,
    form: Form<ConfigUpdateForm>,
    uid: Uuid,
    pool: &State<PgPool>,
) -> Result<Redirect, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let form = form.into_inner();
    let mut device = DeviceORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await?;

    device.name = form.name;
    device.queue = Some(form.queue);
    device.driver_id = form.driver_id;
    device.serial = form.serial;
    device.update_all_fields(&mut *conn).await?;

    for (config_uid, value) in form.configs.into_iter() {
        let config = DeviceConfigORM::select()
            .where_("uid = ? and device = ?")
            .bind(config_uid)
            .bind(uid)
            .fetch_one(&mut *conn)
            .await?;

        config
            .update_partial()
            .value(value)
            .updated_at(unix_epoch_seconds())
            .update(&mut *conn)
            .await?;
    }
    Ok(Redirect::to(format!("/devices/{}/info", uid)))
}

#[get("/<uid>/configuration")]
pub async fn device_config(
    guard: Guard,
    rt: &State<NativeManager>,
    uid: Uuid,
    pool: &State<PgPool>,
) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    // I need to query device, configs, get print queue, and lookup connected prints.
    let device = DeviceORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    let queues = PrintQueueORM::select()
        .fetch_all(&mut *conn)
        .await
        .expect("failed to fetch queues");

    let drivers = rt.index_drivers().await.unwrap();
    let configs = DeviceConfigORM::select()
        .where_("device = ?")
        .bind(uid)
        .fetch_all(&mut *conn)
        .await
        .unwrap();

    RawHtml(Template::render(
        "pages/devices/configuration",
        context! { device, drivers, configs, queues },
    ))
}

#[get("/<uid>/captures")]
pub async fn device_captures(guard: Guard, uid: Uuid, pool: &State<PgPool>) -> RawHtml<Template> {
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
pub async fn device_files(
    guard: Guard,
    rt: &State<NativeManager>,
    uid: Uuid,
    pool: &State<PgPool>,
) -> Result<RawHtml<Template>, OpenFabErr> {
    let mut conn = pool.acquire().await.unwrap();
    // I need to query device, configs, get print queue, and lookup connected prints.
    let device = DeviceORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let mut library = None;
    if let Some(devlib) = device.library {
        library = Some(LibraryORM::get_library(pool, devlib).await?);
    }
    let vals = DeviceFileORM::reconcile(rt.runtime(), device.uid, pool)
        .await
        .expect("failed to list using reconciliaton");
    Ok(RawHtml(Template::render(
        "pages/devices/files",
        context! { device, library, files: vals },
    )))
}
/*
#[get("/<device>/files/<file>/asset")]
pub async fn asset_match(pool: &State<PgPool>, device: Uuid, file: Uuid) -> Result<Redirect, OpenFabErr> {
    let file = DeviceFileORM::get_file(pool, file).await?;
    let asset = LibraryORM::get_asset_by_hash(pool, &file.hash_alg, &file.hash).await?;
    let route = format!("/library/assets/{}", asset.uid);
    Ok(Redirect::to(route))
}*/

#[get("/<device>/files/add-to-device?<asset>")]
pub async fn add_asset_to_device(
    guard: Guard,
    rt: &State<NativeManager>,
    store: &State<ObjectStore>,
    pool: &State<PgPool>,
    device: Uuid,
    asset: Uuid,
) -> Result<Redirect, OpenFabErr> {
    let device = DeviceORM::get_device(pool, device).await?;
    let asset = LibraryORM::get_asset(pool, asset).await?;
    let mut file = device
        .upload_file(rt.runtime(), store, pool, &asset)
        .await?;
    let mut conn = pool.acquire().await?;
    file.uploaded = true;
    file.update_all_fields(&mut *conn).await?;
    let route = format!("/devices/{}/files", device.uid);
    Ok(Redirect::to(route))
}

/*
#[get("/<uid>/files/<file>/download")]
pub async fn download_device_file(
    rt: &State<NativeManager>,
    uid: Uuid,
    file: Uuid,
    pool: &State<PgPool>,
) -> Result<(), OpenFabErr> {
    let mut conn = pool.acquire().await.unwrap();
    // I need to query device, configs, get print queue, and lookup connected prints.
    let device = DeviceORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    let vals = DeviceFileORM::reconcile(rt.runtime(), device.uid, pool).await.expect("failed to list using reconciliaton");
    RawHtml(Template::render(
        "pages/devices/files",
        context! { device, files: vals },
    ))
}*/

#[derive(FromForm)]
pub struct LocalPrintForm {
    file: String,
}

#[post("/<uid>/start-local", data = "<form>")]
pub async fn start_local_print(
    guard: Guard,
    rt: &State<NativeManager>,
    pool: &State<PgPool>,
    uid: Uuid,
    form: Form<LocalPrintForm>,
) -> Result<Redirect, OpenFabErr> {
    let device = DeviceORM::get_device(pool, uid).await?;
    let file = form.into_inner().file;
    let mut path = PathBuf::new();
    path.push(file);
    rt.runtime()
        .get_driver(&device.driver_id)?
        .lock()
        .await
        .start_print(&device.serial, &path)
        .await
        .expect("failed to start print");
    Ok(Redirect::to(format!("/devices/{}/status", device.uid)))
}
/*
#[derive(Debug, Clone, FromForm)]
pub struct DeviceFileUpload {
    library: Uuid,
}

#[post("/<uid>/files/<file>/upload", data = "<upload>")]
pub async fn upload_to_library(rt: &State<NativeManager>, pool: &State<PgPool>, uid: Uuid, file: Uuid, upload: Form<DevceFileUpload>) -> Result<Redirect, OpenFabErr> {
    //let file_orm = DeviceORM::get_file(pool, &rt.runtime(), uid).await?;
    // Download the files contents + metadata
    // rt.download_file()
    //file.uploaded = true;
    Ok(Redirect::to(format!("/devices/files")))
}
*/
#[get("/<uid>/logs")]
pub async fn device_logs(
    guard: Guard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<RawHtml<Template>, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let device = DeviceORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await?;

    let logs = DeviceLogORM::select()
        .where_("device = ?")
        .bind(uid)
        .order_desc("date")
        .fetch_all(&mut *conn)
        .await?;

    Ok(RawHtml(Template::render(
        "pages/devices/logs",
        context! { logs, device },
    )))
}

use rocket::http::Status;

#[get("/<uid>/status")]
pub async fn live_status(
    guard: Guard,
    pool: &State<PgPool>,
    manager: &State<NativeManager>,
    socket: WebSocket,
    uid: Uuid,
) -> Result<Channel<'static>, Status> {
    // 1. Acquire connection and fetch device, returning 404 if missing
    let mut conn = pool
        .acquire()
        .await
        .map_err(|_| Status::InternalServerError)?;
    let device = DeviceORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let serial = device.serial;

    // 2. Try subscribing, returning 400 Bad Request if it fails
    let rx = match manager.subscribe(&device.driver_id, &serial).await {
        Ok(rx) => rx,
        Err(e) => {
            // log the error
            eprintln!("failed to connect to device: {e}");
            DeviceLogORM::error(device.uid, e.to_string(), None)
                .insert(&mut *conn)
                .await
                .map_err(|e| Status::InternalServerError);
            return Err(Status::BadRequest);
        }
    };

    // 3. Hand over to the WebSocket channel only if everything succeeded
    Ok(socket.channel(move |mut stream| {
        Box::pin(async move {
            let mut rx = rx;
            loop {
                if let Some(Some(_message)) = stream.next().now_or_never() {
                    // handle incoming client messages if needed
                }

                match rx.recv().await {
                    Ok(event) => {
                        let _ = stream
                            .send(
                                serde_json::to_string(&event)
                                    .expect("failed to serialize event")
                                    .into(),
                            )
                            .await;
                    }
                    Err(e) => {
                        let _ = stream
                            .send(
                                serde_json::to_string(&Event::Err(e.to_string()))
                                    .expect("failed to serialize event")
                                    .into(),
                            )
                            .await;
                        break;
                    }
                }
            }
            Ok(())
        })
    }))
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
        device_files,
        live_status,
        device_logs,
        edit_device_config,
        start_local_print,
        add_asset_to_device,
        //asset_match,
    ]
}
