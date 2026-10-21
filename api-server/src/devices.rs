use crate::errors::ErrorResponse;
use crate::utils::ApiGuard;
use ormlite::postgres::PgPool;
use rocket::{State, delete, get, post, put, serde::json::Json};
use rocket_okapi::{openapi, openapi_get_routes_spec};
use serverlib::devices::{DeviceLog, DeviceLogORM, DeviceModelORM, DeviceORM, DeviceStatus};
use serverlib::prints::PrintJobInstance;
use serverlib::setup::NativeManager;
use openfab_drivers::runtime::DriverRuntime;
use uuid::Uuid;

#[openapi(tag = "Devices")]
#[get("/models?<page>")]
pub(crate) async fn list_device_models(
    pool: &State<PgPool>,
    page: Option<i64>,
) -> Result<Json<Vec<DeviceModelORM>>, ErrorResponse> {
    Ok(Json(DeviceModelORM::list(pool).await?))
}

#[openapi(tag = "Devices")]
#[get("/?<page>")]
pub(crate) async fn list_devices(
    guard: ApiGuard,
    pool: &State<PgPool>,
    page: Option<i64>,
) -> Result<Json<Vec<DeviceORM>>, ErrorResponse> {
    Ok(Json(DeviceORM::list(pool).await?))
}

#[openapi(tag = "Devices")]
#[get("/<uid>/info")]
pub(crate) async fn get_device(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<DeviceORM>, ErrorResponse> {
    Ok(Json(DeviceORM::get_device(pool, uid).await?))
}

#[openapi(tag = "Devices")]
#[get("/<uid>/logs")]
pub(crate) async fn get_device_logs(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<Vec<DeviceLog>>, ErrorResponse> {
    Ok(Json(DeviceORM::logs(pool, uid).await?))
}

#[openapi(tag = "Devices")]
#[post("/create")]
pub(crate) async fn create_device(
    guard: ApiGuard,
    pool: &State<PgPool>,
) -> Result<Json<DeviceORM>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("create_device"))
}

#[openapi(tag = "Devices")]
#[post("/<uid>/update")]
pub(crate) async fn update_device(uid: Uuid) -> Result<Json<DeviceORM>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("update_device"))
}

#[openapi(tag = "Devices")]
#[delete("/<uid>/delete")]
pub(crate) async fn delete_device(uid: Uuid) -> Result<Json<DeviceORM>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("delete_device"))
}

/// pauses the current print on the machine
#[openapi(tag = "Devices")]
#[put("/<uid>/pause")]
pub(crate) async fn pause_print(
    guard: ApiGuard,
    rt: &State<NativeManager>,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<(), ErrorResponse> {
    let device = DeviceORM::get_device(pool, uid)
        .await
        .expect("failed to get device");
    rt.runtime()
        .get_driver(&device.driver_id)
        .expect("driver error")
        .lock()
        .await
        .pause_print(&device.serial)
        .await
        .expect("failed to stop print");
    Err(ErrorResponse::unimplemented("stop_print"))
}

/// pauses the current print on the machine.
#[openapi(tag = "Devices")]
#[put("/<uid>/resume")]
pub(crate) async fn resume_print(
    guard: ApiGuard,
    rt: &State<NativeManager>,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<PrintJobInstance>, ErrorResponse> {
    let device = DeviceORM::get_device(pool, uid)
        .await
        .expect("failed to get device");
    rt.runtime()
        .get_driver(&device.driver_id)
        .expect("driver error")
        .lock()
        .await
        .resume_print(&device.serial)
        .await
        .expect("failed to stop print");
    Err(ErrorResponse::unimplemented("stop_print"))
}

/// stops the current print on the machine
#[openapi(tag = "Devices")]
#[put("/<uid>/stop")]
pub(crate) async fn stop_print(
    guard: ApiGuard,
    rt: &State<NativeManager>,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<PrintJobInstance>, ErrorResponse> {
    let device = DeviceORM::get_device(pool, uid)
        .await
        .expect("failed to get device");
    rt.runtime()
        .get_driver(&device.driver_id)
        .expect("driver error")
        .lock()
        .await
        .stop_print(&device.serial)
        .await
        .expect("failed to stop print");
    Err(ErrorResponse::unimplemented("stop_print"))
}

pub fn get_routes() -> (Vec<rocket::Route>, okapi::openapi3::OpenApi) {
    openapi_get_routes_spec![
        list_devices,
        get_device,
        create_device,
        update_device,
        delete_device,
        get_device_logs,
        pause_print,
        resume_print,
        stop_print,
        list_device_models,
    ]
}
