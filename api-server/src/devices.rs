use rocket::{delete, get, post, put};
use rocket_okapi::{openapi, openapi_get_routes};
use uuid::Uuid;

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

pub fn get_routes() -> Vec<rocket::Route> {
    openapi_get_routes![
        list_devices,
        get_device,
        create_device,
        update_device,
        delete_device,
    ]
}
