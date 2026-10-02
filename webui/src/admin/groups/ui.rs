use crate::admin::groups::forms::GroupForm;
use rocket::{
    State, form::Form, get, post, response::Redirect, response::content::RawHtml, routes,
};
use rocket_dyn_templates::{Template, context};

#[get("/groups")]
pub fn index() -> RawHtml<Template> {
    let context = context! {};
    RawHtml(Template::render("pages/admin/groups/index", &context))
}

#[get("/groups/create")]
pub fn create_page() -> RawHtml<Template> {
    let context = context! {};
    RawHtml(Template::render("pages/admin/groups/create", &context))
}

#[post("/groups/create", data = "<form>")]
pub fn create_group(form: Form<GroupForm>) -> Redirect {
    unimplemented!();
}

pub fn get_routes() -> Vec<rocket::Route> {
    routes![index, create_page, create_group]
}
