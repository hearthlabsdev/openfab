use crate::accounts::{RegisterForm, LoginForm};
use rocket_dyn_templates::{context, Template};
use rocket::response::content::RawHtml;
use rocket::{get, post, routes, Route};
use rocket::form::Form;
use rocket::response::Redirect;

#[get("/login")]
pub fn login_page() -> RawHtml<Template> {
    RawHtml(Template::render("pages/accounts/login", context! {}))
}

#[post("/login", data = "<login_form>")]
pub fn login(login_form: Form<LoginForm>) -> Redirect {
    // Handle login logic here
    unimplemented!()
}

#[get("/register")]
pub fn register_page() -> RawHtml<Template> {
    RawHtml(Template::render("pages/accounts/register", context! {}))
}

#[post("/register", data = "<register_form>")]
pub fn register(register_form: Form<RegisterForm>) -> Redirect {
    // Handle registration logic here
    unimplemented!()
}

pub fn get_routes() -> Vec<Route> {
    routes![
        login_page,
        login,
        register_page,
        register,
    ]
}