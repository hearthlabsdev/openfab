use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use rocket::{Route, State, get, post, routes};
use rocket_dyn_templates::{Template, context};

use crate::accounts::User;
use crate::admin::users::forms::UserForm;
use crate::utils::Guard;

#[get("/users")]
pub async fn index(guard: Guard, pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let users = User::select().fetch_all(&mut *conn).await.unwrap();

    RawHtml(Template::render(
        "pages/admin/users/index",
        context! { users },
    ))
}

#[post("/users/create", data = "<form>")]
pub async fn create_user(form: Form<UserForm<'_>>) -> Redirect {
    unimplemented!();
}

pub fn get_routes() -> Vec<Route> {
    routes![index, create_user]
}
