use rocket_dyn_templates::{context, Template};
use rocket::response::content::RawHtml;
use rocket::{get, post, routes, Route, State};
use rocket::form::Form;
use rocket::response::Redirect;
use ormlite::postgres::PgPool;
use ormlite::Model;

use crate::utils::Guard;
use crate::admin::users::forms::UserForm;
use crate::accounts::User;

#[get("/users")]
pub async fn index(guard: Guard, pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let users = User::select()
        .fetch_all(&mut *conn)
        .await
        .unwrap();

    RawHtml(Template::render("pages/admin/users/index", context! { users }))
}

#[post("/users/create", data = "<form>")]
pub async fn create_user(form: Form<UserForm<'_>>) -> Redirect {
    unimplemented!();
}

pub fn get_routes() -> Vec<Route> {
    routes![index, create_user]
}