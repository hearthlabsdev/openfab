use crate::errors::ErrorResponse;
use crate::utils::ApiGuard;
use openfab_sdk::models::{CreateAsset, UploadObject};
use rocket::serde::json::Json;
use rocket::fs::TempFile;
use rocket::{State, delete, get, post, put};
use rocket_okapi::{openapi, openapi_get_routes_spec};
use serverlib::library::{AssetORM, LibraryORM};
use uuid::Uuid;
use serverlib::library::ObjectStore;
use serverlib::utils::unix_epoch_seconds;

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
#[get("/assets/<uid>")]
pub(crate) async fn get_asset(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<AssetORM>, ErrorResponse> {
    Ok(Json(LibraryORM::get_asset(pool, uid).await?))
}

#[openapi(tag = "Library")]
#[get("/<uid>/assets?<page>")]
pub(crate) async fn list_assets(
    guard: ApiGuard,
    pool: &State<PgPool>,
    uid: Uuid,
    page: Option<i64>,
) -> Result<Json<Vec<AssetORM>>, ErrorResponse> {
    Ok(Json(LibraryORM::list_assets(pool, uid, page).await?))
}

#[openapi(tag = "Library")]
#[post("/assets/create", data = "<asset>")]
pub(crate) async fn create_asset(
    guard: ApiGuard,
    pool: &State<PgPool>,
    asset: Json<CreateAsset>,
) -> Result<Json<UploadObject>, ErrorResponse> {
    let asset = asset.into_inner();
    let asset = AssetORM::blank(guard.owner(), asset.library, asset.name, asset.public, asset.license);
    let mut conn = pool.acquire().await?;
    let asset = asset.insert(&mut *conn).await?;
    Ok(Json(UploadObject {
        uid: asset.uid,
        put_url: format!("/api/library/assets/{}/upload", asset.uid),
    }))
}

#[openapi(tag = "Library")]
#[put("/assets/<uid>/upload", data = "<file>")]
pub(crate) async fn upload_asset(
    guard: ApiGuard,
    pool: &State<PgPool>,
    store: &State<ObjectStore>,
    uid: Uuid,
    file: TempFile<'_>,
) -> Result<Json<AssetORM>, ErrorResponse> {
    let mut conn = pool.acquire().await?;
    let mut asset = LibraryORM::get_asset(pool, uid).await?;
    let info = store.upload_temp_file(&asset.key, &file).await.expect("upload failure");
    asset.size = info.size;
    asset.hash_alg = info.hash_alg;
    asset.hash = info.hash;
    asset.mime = info.mime;
    asset.updated_at = unix_epoch_seconds();
    let asset = asset.update_all_fields(&mut *conn).await?;
    Ok(Json(asset))
}

#[openapi(tag = "Library")]
#[delete("/assets/<uid>/delete")]
pub(crate) async fn delete_asset(
    guard: ApiGuard,
    store: &State<ObjectStore>,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Json<AssetORM>, ErrorResponse> {
    Ok(Json(AssetORM::delete(store, pool, uid).await?))
}

#[openapi(tag = "Library")]
#[post("/create")]
pub(crate) async fn create_library(guard: ApiGuard) -> Result<Json<LibraryORM>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("create_library"))
}

#[openapi(tag = "Library")]
#[put("/<uid>/update")]
pub(crate) async fn update_asset(
    guard: ApiGuard,
    uid: Uuid,
) -> Result<Json<AssetORM>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("update_asset"))
}

#[openapi(tag = "Library")]
#[delete("/<uid>/delete")]
pub(crate) async fn delete_library(
    guard: ApiGuard,
    uid: Uuid,
) -> Result<Json<LibraryORM>, ErrorResponse> {
    Err(ErrorResponse::unimplemented("delete_library"))
}

pub fn get_routes() -> (Vec<rocket::Route>, okapi::openapi3::OpenApi) {
    openapi_get_routes_spec![
        list_libraries,
        create_library,
        delete_library,
        list_assets,
        get_asset,
        create_asset,
        update_asset,
        delete_asset,
        upload_asset,
    ]
}
