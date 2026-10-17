use rocket::serde::json::Json;
use rocket::{State, delete, get, post, put};
use rocket_okapi::{openapi, openapi_get_routes};
use serverlib::library::{AssetORM, LibraryORM};
use uuid::Uuid;

use ormlite::Model;
use ormlite::postgres::PgPool;

#[openapi(tag = "Library")]
#[get("/library")]
pub(crate) async fn list_libraries(pool: &State<PgPool>) -> Json<Vec<LibraryORM>> {
    Json(
        LibraryORM::list(pool)
            .await
            .expect("failed to list libraries"),
    )
}

#[openapi(tag = "Library")]
#[get("/library/assets/<uid>")]
pub(crate) async fn get_asset(pool: &State<PgPool>, uid: Uuid) -> Json<AssetORM> {
    Json(
        LibraryORM::get_asset(pool, uid)
            .await
            .expect("failed to get asset"),
    )
}

#[openapi(tag = "Library")]
#[get("/library/<uid>/assets?<page>")]
pub(crate) async fn list_assets(
    pool: &State<PgPool>,
    uid: Uuid,
    page: Option<i64>,
) -> Json<Vec<AssetORM>> {
    Json(
        LibraryORM::list_assets(pool, uid, page)
            .await
            .expect("failed to list assets"),
    )
}

#[openapi(tag = "Library")]
#[post("/library/assets")]
pub(crate) async fn create_asset(pool: &State<PgPool>) {
    unimplemented!();
}

#[openapi(tag = "Library")]
#[delete("/library/assets/<uid>/delete")]
pub(crate) async fn delete_asset(pool: &State<PgPool>, uid: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Library")]
#[post("/library/create")]
pub(crate) async fn create_library() {
    unimplemented!();
}

#[openapi(tag = "Library")]
#[put("/library/<uid>/update")]
pub(crate) async fn update_asset(uid: Uuid) {
    unimplemented!();
}

#[openapi(tag = "Library")]
#[delete("/library/<uid>/delete")]
pub(crate) async fn delete_library(uid: Uuid) {
    unimplemented!();
}

pub fn get_routes() -> Vec<rocket::Route> {
    openapi_get_routes![
        list_libraries,
        create_library,
        delete_library,
        list_assets,
        get_asset,
        create_asset,
        update_asset,
        delete_asset,
    ]
}
