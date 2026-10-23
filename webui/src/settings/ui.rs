use crate::devices::DeviceModelORM;
use crate::errors::OpenFabErr;
use crate::library::{AssetORM, ObjectStore};
use crate::settings::DeviceModelInfo;
use crate::settings::SettingsORM;
use crate::settings::materials::FilamentORM;
use crate::ui::themes::ThemeORM;
use crate::utils::Guard;
use crate::utils::unix_epoch_seconds;
use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::form::FromForm;
use rocket::response::{Redirect, content::RawHtml};
use rocket::{Route, State, delete, form::Form, get, post, put, routes};
use rocket_dyn_templates::{Template, context};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct CreateStyleForm {
    pub name: String,

    pub background: String,
    pub surface: String,
    pub surface_hover: String,
    pub surface_header: String,

    pub border: String,
    pub border_subtle: String,

    pub text: String,
    pub text_strong: String,
    pub text_muted: String,

    pub accent: String,
}
impl CreateStyleForm {
    pub fn into_orm(self) -> ThemeORM {
        ThemeORM {
            uid: Uuid::new_v4(),
            name: self.name,
            background: self.background,
            surface: self.surface,
            surface_hover: self.surface_hover,
            surface_header: self.surface_header,

            border: self.border,
            border_subtle: self.border_subtle,
            text: self.text,
            text_strong: self.text_strong,
            text_muted: self.text_muted,
            accent: self.accent,
            created_at: unix_epoch_seconds(),
        }
    }
}

#[get("/devices?<page>")]
pub async fn devices(
    guard: Guard,
    pool: &State<PgPool>,
    store: &State<ObjectStore>,
    page: Option<isize>,
) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let mut devices = Vec::new();
    let page_size: usize = 20;
    let page: usize = (page.unwrap_or(1) - 1).abs().try_into().unwrap_or(0);
    for orm in DeviceModelORM::select()
        .limit(20)
        .offset(page * page_size)
        .fetch_all(&mut *conn)
        .await
        .unwrap()
        .into_iter()
    {
        devices.push(
            DeviceModelInfo::from_device_model_orm(pool, store, orm)
                .await
                .unwrap(),
        );
    }

    RawHtml(Template::render(
        "pages/settings/devlibrary/index",
        context! { device_models: devices, page: page + 1 },
    ))
}

#[get("/devices/<uid>")]
pub async fn device(
    guard: Guard,
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

    let info = DeviceModelInfo::from_device_model_orm(pool, store, device)
        .await
        .unwrap();
    println!("info: {:?}", info);
    RawHtml(Template::render(
        "pages/settings/devlibrary/device",
        context! { device: info },
    ))
}

#[get("/materials/filaments?<page>")]
pub async fn filaments(
    guard: Guard,
    pool: &State<PgPool>,
    page: Option<usize>,
) -> RawHtml<Template> {
    let page_size = 100;
    let page = (page.unwrap_or(1) - 1);
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
pub async fn filament(guard: Guard, pool: &State<PgPool>, uid: Uuid) -> RawHtml<Template> {
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
pub async fn index(guard: Guard) -> RawHtml<Template> {
    RawHtml(Template::render("pages/settings/index", context! {}))
}

#[get("/styles")]
pub async fn styles(guard: Guard, pool: &State<PgPool>) -> Result<RawHtml<Template>, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let themes = ThemeORM::select().fetch_all(&mut *conn).await?;

    Ok(RawHtml(Template::render(
        "pages/settings/styles/index",
        context! { themes },
    )))
}

#[get("/styles/create")]
pub async fn create_style_page(guard: Guard) -> Result<RawHtml<Template>, OpenFabErr> {
    Ok(RawHtml(Template::render(
        "pages/settings/styles/create",
        context! {},
    )))
}

#[post("/styles/create", data = "<form>")]
pub async fn create_style(
    guard: Guard,
    pool: &State<PgPool>,
    form: Form<CreateStyleForm>,
) -> Result<Redirect, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let form = form.into_inner().into_orm().insert(&mut *conn).await?;
    Ok(Redirect::to(format!("/settings/styles/{}", form.uid)))
}

#[post("/styles/<uid>/apply")]
pub async fn apply_style(
    guard: Guard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<Redirect, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let mut setting = SettingsORM::select()
        .where_("key = 'theme'")
        .fetch_one(&mut *conn)
        .await?;
    setting.value = uid.to_string();
    setting.update_all_fields(&mut *conn).await?;
    Ok(Redirect::to("/settings/styles"))
}

#[get("/styles/<uid>")]
pub async fn get_style(
    guard: Guard,
    pool: &State<PgPool>,
    uid: Uuid,
) -> Result<RawHtml<Template>, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let theme = ThemeORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await?;
    Ok(RawHtml(Template::render(
        "pages/settings/styles/theme",
        context! { theme },
    )))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        devices,
        device,
        filament,
        filaments,
        index,
        styles,
        create_style,
        create_style_page,
        get_style,
        apply_style
    ]
}
