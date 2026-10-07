use crate::errors::OpenFabErr;
use crate::library::ObjectStore;
use crate::library::{AssetORM, LibraryORM};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub uid: Uuid,
    pub name: String,
    pub mime: String,
    /// a presigned get url to display in the webui
    pub url: String,
    pub library: Uuid,
    /// Size of the file in bytes.
    pub size: i64,

    pub created_at: i64,
    pub updated_at: i64,
    /// Last time this asset was used in a print job or queue.
    pub last_used_at: Option<i64>,

    /// Whether this asset is publicly accessible (e.g. for embedding or sharing)
    pub public: bool,
    /// Optional license or usage terms (e.g. "CC-BY-SA 4.0")
    pub license: Option<String>,
}

impl AssetInfo {
    pub async fn from_asset_orm(orm: AssetORM, store: &ObjectStore) -> Result<Self, OpenFabErr> {
        Ok(Self {
            uid: orm.uid,
            name: orm.name,
            mime: orm.mime,
            library: orm.library,
            url: store.presigned_get_url(&orm.key, 60 * 60).await?,
            size: orm.size,
            created_at: orm.created_at,
            updated_at: orm.updated_at,
            last_used_at: orm.last_used_at,
            public: orm.public,
            license: orm.license,
        })
    }
}

#[derive(Debug, FromForm)]
pub struct AssetForm<'r> {
    pub name: String,
    pub library: Uuid,
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
            library: self.library,
            key: filename,
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
pub async fn index(store: &State<ObjectStore>, pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let assets: Option<Uuid> = None;
    let libraries = LibraryORM::select()
        .where_("public = true")
        .fetch_all(&mut *conn)
        .await
        .expect("failed to fetch libraries");
    println!("libraries length: {}", libraries.len());

    RawHtml(Template::render(
        "pages/library/index",
        context! { assets, libraries },
    ))
}

#[get("/<library>?<page>")]
pub async fn index_assets(
    store: &State<ObjectStore>,
    pool: &State<PgPool>,
    library: Uuid,
    page: Option<isize>,
) -> RawHtml<Template> {
    let page_size: usize = 20;
    let page: usize = (page.unwrap_or(1) - 1).abs().try_into().unwrap_or(0);
    let mut conn = pool.acquire().await.unwrap();
    let mut assets = Vec::new();
    for asset in AssetORM::select()
        .where_("library = ?")
        .bind(library)
        .order_desc("created_at")
        .limit(page_size)
        .offset(page * page_size)
        .fetch_all(&mut *conn)
        .await
        .unwrap()
        .into_iter()
    {
        assets.push(AssetInfo::from_asset_orm(asset, &store).await.unwrap());
    }
    let libraries = LibraryORM::select()
        .where_("public = true")
        .fetch_all(&mut *conn)
        .await
        .expect("failed to fetch libraries");
    println!("assets len: {}", assets.len());
    RawHtml(Template::render(
        "pages/library/index",
        context! { assets, libraries, library, page: page + 1 },
    ))
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

#[get("/assets/<uid>/download")]
pub async fn download(pool: &State<PgPool>, store: &State<ObjectStore>, uid: Uuid) -> Redirect {
    let mut conn = pool.acquire().await.unwrap();
    let asset = AssetORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    let url = store.presigned_get_url(&asset.key, 60 * 60).await.unwrap();
    Redirect::to(url)
}

#[get("/assets/<uid>")]
pub async fn get_asset(
    pool: &State<PgPool>,
    store: &State<ObjectStore>,
    uid: Uuid,
) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let asset = AssetORM::select()
        .where_("uid = ?")
        .bind(uid)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let info = AssetInfo::from_asset_orm(asset, &store).await.unwrap();
    RawHtml(Template::render(
        "pages/library/asset",
        context! { asset: info },
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        create_asset,
        create_form,
        index,
        index_assets,
        download,
        get_asset
    ]
}
