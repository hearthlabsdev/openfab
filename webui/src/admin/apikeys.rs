use ormlite::{Model, postgres::PgPool};
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use rocket::route::Route;
use rocket::{
    State,
    form::{Form, FromForm},
    get, post, routes,
};
use rocket_dyn_templates::{Template, context};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounts::User;
use crate::utils::Guard;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "api_keys")]
pub struct ApiKeyORM {
    #[ormlite(primary_key)]
    uid: Uuid,
    keyid: String,
    owner: Uuid,
    /// the user's who's permissions this key is allowed to inherit
    account: Option<Uuid>,
    access_key: String,
    // expiration as unix epoch
    expires: i64,
    created: i64,
}

#[get("/keys")]
async fn index(guard: Guard, pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let user = User::select()
        .where_("subject = ?")
        .bind(guard.claims.sub.to_string())
        .fetch_one(&mut *conn)
        .await
        .expect("failed to fetch current user");

    let keys = ApiKeyORM::select()
        .where_("owner = ?")
        .bind(user.uid())
        .fetch_all(&mut *conn)
        .await
        .expect("failed to fetch API Keys");

    RawHtml(Template::render(
        "pages/admin/keys/index",
        context! { api_keys: keys },
    ))
}

#[get("/keys/create")]
async fn create_access_key_page(guard: Guard, pool: &State<PgPool>) -> RawHtml<Template> {
    unimplemented!();
}

#[post("/keys/create")]
async fn create_access_key(guard: Guard, pool: &State<PgPool>) -> Redirect {
    Redirect::to("/admin/keys")
}

pub fn get_routes() -> Vec<Route> {
    routes![create_access_key, create_access_key_page, index]
}
