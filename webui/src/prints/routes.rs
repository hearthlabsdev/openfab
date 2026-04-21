use rocket_dyn_templates::{context, Template};
use rocket::response::content::RawHtml;
use rocket::{get, routes, Route};

#[get("/")]
pub async fn index() -> RawHtml<Template> {
    RawHtml(Template::render("pages/prints/queue", context! {}))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        crate::prints::routes::index,
    ]
}