use rocket_dyn_templates::{context, Template};
use rocket::response::content::RawHtml;
use rocket::{get, post, routes, Route};
use crate::admin::users::forms::UserForm;
use rocket::form::Form;
use rocket::response::Redirect;

#[get("/users")]
pub async fn index() -> RawHtml<Template> {
    RawHtml(Template::render("pages/admin/users/index", context! {}))
}

#[post("/users/create", data = "<form>")]
pub async fn create_user(form: Form<UserForm<'_>>) -> Redirect {
    unimplemented!();
}

pub fn get_routes() -> Vec<Route> {
    routes![index, create_user]
}