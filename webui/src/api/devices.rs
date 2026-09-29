use crate::devices::*;
use uuid::Uuid;
use rocket::{get, put, post, delete};
use rocket_okapi::{openapi};

#[openapi(tag = "Devices")]
#[get("/devices")]
pub(crate) async fn list_devices() {
    unimplemented!();
}

#[openapi(tag = "Devices")]
#[get("/devices/<id>")]
pub(crate) async fn get_device(id: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Devices")]
#[post("/devices/create")]
pub(crate) async fn create_device() {
    unimplemented!();
}

#[openapi(tag = "Devices")]
#[post("/devices/update/<id>")]
pub(crate) async fn update_device(id: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Devices")]
#[delete("/devices/delete/<id>")]
pub(crate) async fn delete_device(id: Uuid) {
    unimplemented!();
}