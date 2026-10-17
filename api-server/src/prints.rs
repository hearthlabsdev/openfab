use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::State;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put};
use rocket_okapi::{openapi, openapi_get_routes};
use serverlib::prints::{PrintJob, PrintJobInstance, PrintJobInstanceORM, PrintJobORM, PrintQueueORM};
use uuid::Uuid;

use serverlib::errors::OpenFabErr;

#[openapi(tag = "Prints")]
#[get("/queue/<uid>")]
pub(crate) async fn list_prints(pool: &State<PgPool>, uid: Uuid) -> Json<Vec<PrintJob>> {
    Json(
        PrintQueueORM::list_prints(pool, uid)
            .await
            .expect("failed to list prints"),
    )
}

#[openapi(tag = "Prints")]
#[get("/<uid>")]
pub(crate) async fn get_print(pool: &State<PgPool>, uid: Uuid) -> Json<PrintJob> {
    Json(
        PrintJobORM::get_print(pool, uid)
            .await
            .expect("failed to get print"),
    )
}

#[openapi(tag = "Prints")]
#[post("/create")]
pub(crate) async fn create_print() {
    unimplemented!();
}

#[openapi(tag = "Prints")]
#[post("/update/<uid>")]
pub(crate) async fn update_print(pool: &State<PgPool>, uid: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Prints")]
#[delete("/delete/<uid>")]
pub(crate) async fn delete_print(pool: &State<PgPool>, uid: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Prints")]
#[get("/status/<uid>")]
pub(crate) async fn get_print_status(pool: &State<PgPool>, uid: Uuid) -> Json<PrintJobInstance> {
    Json(
        PrintJobInstanceORM::get_job_instance(pool, uid)
            .await
            .expect("failed to get print status"),
    )
}

#[openapi(tag = "Prints")]
#[put("/<uid>/pause")]
pub(crate) async fn pause_print(pool: &State<PgPool>, uid: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Prints")]
#[put("/<uid>/resume")]
pub(crate) async fn resume_print(pool: &State<PgPool>, uid: Uuid) {}

#[openapi(tag = "Prints")]
#[put("/<uid>/stop")]
pub(crate) async fn stop_print(pool: &State<PgPool>, uid: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Queues")]
#[get("/queue/<uid>/info")]
pub(crate) async fn queue_info(pool: &State<PgPool>, uid: Uuid) -> Json<PrintQueueORM> {
    Json(PrintQueueORM::get_queue(pool, uid)
        .await
        .expect("failed to get print queue"))
}

#[openapi(tag = "Queues")]
#[get("/queue")]
pub(crate) async fn list_queues(pool: &State<PgPool>) -> Json<Vec<PrintQueueORM>> {
    Json(PrintQueueORM::list(pool)
        .await
        .expect("failed to list queues"))
}

pub fn get_routes() -> Vec<rocket::Route> {
    openapi_get_routes![
        list_prints,
        get_print,
        create_print,
        get_print_status,
        pause_print,
        stop_print,
        delete_print,
        queue_info,
        list_queues,
    ]
}
