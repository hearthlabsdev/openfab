use crate::devices::DeviceModelORM;
use crate::library::{AssetORM, ObjectStore};
use crate::settings::materials::FilamentORM;
use crate::settings::DeviceModelInfo;
use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::response::content::RawHtml;
use rocket::{Route, State, delete, form::Form, get, post, put, routes};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[get("/devices")]
pub async fn devices(pool: &State<PgPool>, store: &State<ObjectStore>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let mut devices = Vec::new();
    
    for orm in DeviceModelORM::select()
        .limit(20)
        .fetch_all(&mut *conn)
        .await
        .unwrap().into_iter() {
            devices.push(DeviceModelInfo::from_device_model_orm(pool, store, orm).await.unwrap());
    }

    RawHtml(Template::render(
        "pages/settings/devlibrary/index",
        context! { device_models: devices },
    ))
}

#[get("/devices/<uid>")]
pub async fn device(
    pool: &State<PgPool>,
    store: &State<ObjectStore>,
    uid: Uuid,
) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let device = DeviceModelORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    let info = DeviceModelInfo::from_device_model_orm(pool, store, device).await.unwrap();
    println!("info: {:?}", info);
    RawHtml(Template::render(
        "pages/settings/devlibrary/device",
        context! { device: info },
    ))
}

#[get("/materials/filaments?<page>")]
pub async fn filaments(pool: &State<PgPool>, page: Option<usize>) -> RawHtml<Template> {
    let page_size = 100;
    let page = (page.unwrap_or(1)-1);
    let mut conn = pool.acquire().await.unwrap();
    let filaments = FilamentORM::select()
        .where_("colour is not null and colour != ''")
        .limit(page_size)
        .offset(page * page_size)
        .fetch_all(&mut *conn)
        .await
        .unwrap();

    RawHtml(Template::render(
        "pages/settings/materials/filaments",
        context! { filaments },
    ))
}

#[get("/materials/filaments/<uid>")]
pub async fn filament(pool: &State<PgPool>, uid: Uuid) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let filament = FilamentORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    RawHtml(Template::render(
        "pages/settings/materials/filament",
        context! { filament },
    ))
}

#[get("/")]
pub async fn index() -> RawHtml<Template> {
    RawHtml(Template::render("pages/settings/index", context! {}))
}

pub fn get_routes() -> Vec<Route> {
    routes![devices, device, filament, filaments, index]
}
