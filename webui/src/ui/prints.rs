use rocket_dyn_templates::{context, Template};
use rocket::response::content::RawHtml;
use rocket::{get, routes, Route};

#[get("/")]
pub async fn index() -> RawHtml<Template> {
    RawHtml(Template::render("pages/prints/queue", context! {}))
}

#[get("/history")]
pub async fn history() -> RawHtml<Template> {
    RawHtml(Template::render("pages/prints/history", context! {}))
}

#[get("/create")]
pub async fn create() -> RawHtml<Template> {
    RawHtml(Template::render("pages/prints/create", context! {}))
}

pub fn get_routes() -> Vec<Route> {
    routes![index, history, create]
}