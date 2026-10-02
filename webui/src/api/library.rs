use crate::library::*;
use rocket::{delete, get, post, put};
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Library")]
#[get("/library")]
pub(crate) async fn list_assets() {
    unimplemented!();
}

#[openapi(tag = "Library")]
#[get("/library/<id>")]
pub(crate) async fn get_asset(id: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Library")]
#[post("/library/create")]
pub(crate) async fn create_asset() {
    unimplemented!();
}

#[openapi(tag = "Library")]
#[put("/library/<id>/update")]
pub(crate) async fn update_asset(id: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Library")]
#[delete("/library/<id>/delete")]
pub(crate) async fn delete_asset(id: Uuid) {
    unimplemented!();
}
