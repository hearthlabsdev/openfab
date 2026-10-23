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
use serverlib::admin::apikeys::ApiKeyORM;
use serverlib::errors::AuthErr;
use serverlib::errors::OpenFabErr;
use uuid::Uuid;

use crate::accounts::User;
use crate::utils::Guard;

#[get("/keys")]
async fn index(guard: Guard, pool: &State<PgPool>) -> Result<RawHtml<Template>, OpenFabErr> {
    let mut conn = pool.acquire().await.unwrap();
    let user = User::select()
        .where_("subject = ?")
        .bind(guard.claims.sub.to_string())
        .fetch_one(&mut *conn)
        .await?;

    let keys = ApiKeyORM::list_for_owner(pool, user.uid()).await?;

    Ok(RawHtml(Template::render(
        "pages/admin/keys/index",
        context! { api_keys: keys },
    )))
}

#[get("/keys/create")]
async fn create_access_key_page(
    guard: Guard,
    pool: &State<PgPool>,
) -> Result<RawHtml<Template>, OpenFabErr> {
    unimplemented!();
}

#[post("/keys/create")]
async fn create_access_key(guard: Guard, pool: &State<PgPool>) -> Result<Redirect, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let user = User::select()
        .where_("subject = ?")
        .bind(guard.claims.sub.to_string())
        .fetch_one(&mut *conn)
        .await?;
    ApiKeyORM::generate_for_user(pool, &user).await?;
    Ok(Redirect::to("/admin/keys"))
}

#[post("/keys/<key>/delete")]
async fn delete_api_key(
    guard: Guard,
    pool: &State<PgPool>,
    key: Uuid,
) -> Result<Redirect, OpenFabErr> {
    ApiKeyORM::delete_key(pool, key).await?;
    Ok(Redirect::to("/admin/keys"))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        create_access_key,
        delete_api_key,
        create_access_key_page,
        index
    ]
}
