use crate::utils::unix_epoch_seconds;
use ormlite::{Model, postgres::PgPool};
use rocket::response::{Redirect, content::RawHtml};
use rocket::{
    State,
    form::{Form, FromForm},
    get, post,
    route::Route,
    routes,
};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "bug_reports")]
pub struct BugReportORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub error: String,
    pub route: Option<String>,
    pub description: Option<String>,
    pub submitted: i64,
}

#[derive(Debug, Clone, FromForm)]
pub struct BugReportForm {
    route: Option<String>,
    error: String,
    description: Option<String>,
}

impl BugReportForm {
    pub fn into_bug_report(self) -> BugReportORM {
        BugReportORM {
            uid: Uuid::new_v4(),
            error: self.error,
            route: self.route,
            description: self.description,
            submitted: unix_epoch_seconds(),
        }
    }
}

#[get("/bugreport?<error>&<route>")]
pub async fn bug_report_page(error: String, route: Option<String>) -> RawHtml<Template> {
    RawHtml(Template::render(
        "pages/admin/bugreport",
        context! { error, route },
    ))
}

#[post("/bugreport", data = "<form>")]
pub async fn create_bug_report(pool: &State<PgPool>, form: Form<BugReportForm>) -> Redirect {
    let mut conn = pool.acquire().await.expect("failed to acquire pool lock");
    let result = form
        .into_inner()
        .into_bug_report()
        .insert(&mut *conn)
        .await
        .expect("failed to submit bug report");

    let redirect = result.route.unwrap_or(String::from("/"));
    Redirect::to(redirect)
}

pub fn get_routes() -> Vec<Route> {
    routes![create_bug_report, bug_report_page]
}
