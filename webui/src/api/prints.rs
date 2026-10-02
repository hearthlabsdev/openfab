use crate::prints::*;
use rocket::{delete, get, post, put};
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Prints")]
#[get("/prints")]
pub(crate) async fn list_prints() {
    unimplemented!();
}

#[openapi(tag = "Prints")]
#[get("/prints/<id>")]
pub(crate) async fn get_print(id: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Prints")]
#[post("/prints/create")]
pub(crate) async fn create_print() {
    unimplemented!();
}

#[openapi(tag = "Prints")]
#[post("/prints/update/<id>")]
pub(crate) async fn update_print(id: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Prints")]
#[delete("/prints/delete/<id>")]
pub(crate) async fn delete_print(id: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Prints")]
#[get("/prints/<id>/status")]
pub(crate) fn get_print_status(id: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Prints")]
#[put("/prints/<id>/pause")]
pub(crate) async fn pause_print(id: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Prints")]
#[put("/prints/<id>/resume")]
pub(crate) async fn resume_print(id: Uuid) {}

#[openapi(tag = "Prints")]
#[put("/prints/<id>/stop")]
pub(crate) async fn stop_print(id: Uuid) {
    unimplemented!();
}
