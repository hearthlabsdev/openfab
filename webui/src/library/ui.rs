use crate::library::AssetORM;
use crate::library::ObjectStore;
use crate::utils::Guard;
use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::State;
use rocket::form::{Form, FromForm};
use rocket::fs::TempFile;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use rocket::{Route, get, post, routes};
use rocket_dyn_templates::{Template, context};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, FromForm)]
pub struct AssetForm<'r> {
    pub name: String,
    pub mime: String,
    pub owner: String,
    pub size: i64,
    pub public: bool,
    pub license: Option<String>,
    pub file: TempFile<'r>,
}

impl AssetForm<'_> {
    pub async fn into_asset(self, store: &ObjectStore) -> Result<AssetORM, String> {
        let uid = uuid::Uuid::new_v4();
        let filename = format!("/assets/{}", uid);
        store
            .upload_temp_file(&filename, &self.file)
            .await
            .map_err(|e| e.to_string())?;

        Ok(AssetORM {
            uid,
            name: self.name,
            mime: self.mime,
            owner: uuid::Uuid::parse_str(&self.owner).map_err(|e| e.to_string())?,
            size: self.size,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            last_used_at: None,
            public: self.public,
            license: self.license,
        })
    }
}

#[get("/")]
pub async fn index(pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let assets = AssetORM::select()
        .order_desc("created_at")
        .limit(20)
        .fetch_all(&mut *conn)
        .await
        .unwrap_or_else(|_| vec![]);
    RawHtml(Template::render("pages/library/index", context! { assets }))
}

#[get("/upload")]
pub async fn create_form(guard: Guard, store: &State<ObjectStore>) -> RawHtml<Template> {
    let uid = Uuid::new_v4();
    let key = format!("/assets/{}", uid);
    let url = store.presigned_put_url(&key, 3600).await.unwrap();
    RawHtml(Template::render(
        "pages/library/create",
        context! { uid, url },
    ))
}

#[post("/upload", data = "<asset_form>")]
pub async fn create_asset(
    store: &State<ObjectStore>,
    asset_form: Form<AssetForm<'_>>,
    pool: &State<PgPool>,
) -> Redirect {
    let mut conn = pool.acquire().await.unwrap();
    let asset_form = asset_form.into_inner();
    let asset = asset_form.into_asset(&store).await.unwrap();
    asset.insert(&mut *conn).await.unwrap();
    Redirect::to("/library")
}

pub fn get_routes() -> Vec<Route> {
    routes![create_asset, create_form, index]
}
