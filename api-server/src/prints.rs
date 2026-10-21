use crate::errors::ErrorResponse;
use crate::utils::ApiGuard;
use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::State;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put};
use rocket_okapi::{openapi, openapi_get_routes_spec};
use serverlib::devices::DeviceORM;
use serverlib::library::ObjectStore;
use serverlib::prints::{
    PrintJob, PrintJobInstance, PrintJobInstanceORM, PrintJobORM, PrintQueueORM,
};
use serverlib::setup::NativeManager;
use uuid::Uuid;

use openfab_drivers::runtime::DriverRuntime;
use serverlib::errors::OpenFabErr;

/// gets information about a print
#[openapi(tag = "Prints")]
#[get("/jobs/<uid>/info")]
pub(crate) async fn get_print(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<PrintJob>, ErrorResponse> {
    Ok(Json(PrintJobORM::get_print(pool, uid).await?))
}

#[openapi(tag = "Prints")]
#[post("/create")]
pub(crate) async fn create_print(guard: ApiGuard) -> Result<Json<PrintJob>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("create_print"))
}

#[openapi(tag = "Prints")]
#[get("/jobs/<uid>/instances")]
pub(crate) async fn list_instances(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<Vec<PrintJobInstance>>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("print_job_instances"))
}

#[openapi(tag = "Prints")]
#[post("/jobs/<uid>/start?<device>")]
pub(crate) async fn start_print(
    guard: ApiGuard,
    rt: &State<NativeManager>,
    pool: &State<PgPool>,
    store: &State<ObjectStore>,
    uid: Uuid,
    device: Option<Uuid>,
) -> Result<Json<PrintJobInstance>, ErrorResponse> {
    let mut job = PrintJobORM::admin_get_print(pool, uid)
        .await
        .expect("print failure");
    let instance = job.start(rt.runtime(), device, pool, store).await?;
    Ok(Json(instance))
}

#[openapi(tag = "Prints")]
#[post("/jobs/<uid>/update")]
pub(crate) async fn update_print(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<PrintJob>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("update_print"))
}

#[openapi(tag = "Prints")]
#[delete("/jobs/<uid>/delete")]
pub(crate) async fn delete_print(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<PrintJob>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("delete_print"))
}

#[openapi(tag = "Prints")]
#[get("/jobs/<uid>/status")]
pub(crate) async fn get_print_status(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<PrintJobInstance>, ErrorResponse> {
    Ok(Json(PrintJobInstanceORM::get_instance(pool, uid).await?))
}

/// get info on a specific Queue
#[openapi(tag = "Queues")]
#[get("/queues/<uid>/info")]
pub(crate) async fn queue_info(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<PrintQueueORM>, ErrorResponse> {
    Ok(Json(
        PrintQueueORM::get_queue(pool, uid)
            .await
            .expect("failed to get queue"),
    ))
}

/// lists queues on the server
#[openapi(tag = "Queues")]
#[get("/queues")]
pub(crate) async fn list_queues(
    guard: ApiGuard,
    pool: &State<PgPool>,
) -> Result<Json<Vec<PrintQueueORM>>, ErrorResponse> {
    Ok(Json(PrintQueueORM::list(pool).await?))
}

/// lists the prints in the given queue
#[openapi(tag = "Queues")]
#[get("/queues/<uid>/jobs")]
pub(crate) async fn list_prints(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<Vec<PrintJob>>, ErrorResponse> {
    Ok(Json(PrintQueueORM::list_prints(pool, uid).await?))
}

pub fn get_routes() -> (Vec<rocket::Route>, okapi::openapi3::OpenApi) {
    openapi_get_routes_spec![
        list_prints,
        get_print,
        create_print,
        get_print_status,
        delete_print,
        queue_info,
        list_queues,
        list_instances,
        start_print,
    ]
}
