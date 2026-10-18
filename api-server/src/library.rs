use crate::errors::ErrorResponse;
use crate::utils::ApiGuard;
use rocket::serde::json::Json;
use rocket::{State, delete, get, post, put};
use rocket_okapi::{openapi, openapi_get_routes};
use serverlib::library::{AssetORM, LibraryORM};
use uuid::Uuid;

use ormlite::Model;
use ormlite::postgres::PgPool;

#[openapi(tag = "Library")]
#[get("/library")]
pub(crate) async fn list_libraries(
    guard: ApiGuard,
    pool: &State<PgPool>,
) -> Result<Json<Vec<LibraryORM>>, ErrorResponse> {
    Ok(Json(LibraryORM::list(pool).await?))
}

#[openapi(tag = "Library")]
#[get("/library/assets/<uid>")]
pub(crate) async fn get_asset(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<AssetORM>, ErrorResponse> {
    Ok(Json(LibraryORM::get_asset(pool, uid).await?))
}

#[openapi(tag = "Library")]
#[get("/library/<uid>/assets?<page>")]
pub(crate) async fn list_assets(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
    page: Option<i64>,
) -> Result<Json<Vec<AssetORM>>, ErrorResponse> {
    Ok(Json(LibraryORM::list_assets(pool, uid, page).await?))
}

#[openapi(tag = "Library")]
#[post("/library/assets")]
pub(crate) async fn create_asset(
    guard: ApiGuard,
    pool: &State<PgPool>,
) -> Result<Json<AssetORM>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("create_asset"))
}

#[openapi(tag = "Library")]
#[delete("/library/assets/<uid>/delete")]
pub(crate) async fn delete_asset(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<AssetORM>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("delete_asset"))
}

#[openapi(tag = "Library")]
#[post("/library/create")]
pub(crate) async fn create_library(guard: ApiGuard) -> Result<Json<LibraryORM>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("create_library"))
}

#[openapi(tag = "Library")]
#[put("/library/<uid>/update")]
pub(crate) async fn update_asset(
    guard: ApiGuard,
    uid: Uuid,
) -> Result<Json<AssetORM>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("update_asset"))
}

#[openapi(tag = "Library")]
#[delete("/library/<uid>/delete")]
pub(crate) async fn delete_library(
    guard: ApiGuard,
    uid: Uuid,
) -> Result<Json<LibraryORM>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("delete_library"))
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
