use crate::admin::groups::forms::GroupForm;
use crate::errors::OpenFabErr;
use ormlite::{Model, postgres::PgPool};
use rocket::{
    State, form::Form, get, post, response::Redirect, response::content::RawHtml, routes,
};
use rocket_dyn_templates::{Template, context};
use serverlib::admin::groups::GroupORM;

#[get("/groups")]
pub async fn index(pool: &State<PgPool>) -> Result<RawHtml<Template>, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let groups = GroupORM::select().fetch_all(&mut *conn).await?;
    let context = context! { groups };
    Ok(RawHtml(Template::render(
        "pages/admin/groups/index",
        &context,
    )))
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
