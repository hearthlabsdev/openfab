use rocket_dyn_templates::{context, Template};
use rocket::response::content::RawHtml;
use rocket::{get, routes, Route, post};
use crate::library::AssetORM;
use rocket::State;
use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::form::{Form, FromForm};
use rocket::fs::TempFile;
use crate::library::ObjectStore;
use rocket::response::Redirect;
use serde::{Serialize, Deserialize};

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
        let filename = format!("assets/{}", uid);
        //self.file.persist_to(&filename).await.map_err(|e| e.to_string())?;
        store.upload_temp_file(&filename, &self.file).await.map_err(|e| e.to_string())?;
        //std::fs::remove_file(&filename).map_err(|e| e.to_string())?;

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

#[get("/create")]
pub async fn create_form() -> RawHtml<Template> {
    RawHtml(Template::render("pages/library/create", context! {}))
}

#[post("/create", data = "<asset_form>")]
pub async fn create_asset(store: &State<ObjectStore>, asset_form: Form<AssetForm<'_>>, pool: &State<PgPool>) -> Redirect {
    let mut conn = pool.acquire().await.unwrap();
    let asset = asset_form.into_inner().into_asset(&store).await.unwrap();
    asset.insert(&mut *conn).await.unwrap();
    Redirect::to("/library")
}

pub fn get_routes() -> Vec<Route> {
    routes![create_asset, create_form, index]
}