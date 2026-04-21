use rocket_dyn_templates::{context, Template};
use rocket::response::content::RawHtml;
use rocket::{get, routes, Route};

#[get("/")]
pub fn index() -> RawHtml<Template> {
    let context = context! {};
    RawHtml(Template::render("pages/devices/index", &context))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        crate::devices::routes::index,
    ]
}