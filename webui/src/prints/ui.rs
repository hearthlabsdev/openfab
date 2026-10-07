use crate::accounts::User;
use crate::devices::DeviceORM;
use crate::prints::AssetORM;
use crate::prints::forms::PrintForm;
use crate::prints::{PrintJobORM, PrintQueueORM};
use crate::utils::Guard;
use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::response::content::RawHtml;
use rocket::{Route, State, form::Form, get, post, response::Redirect, routes};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[get("/queues/<queue>")]
async fn jobs(pool: &State<PgPool>, queue: Uuid) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let queue = PrintQueueORM::select()
        .where_("uid = ?")
        .bind(queue)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    let prints = PrintJobORM::select()
        .join(PrintJobORM::user())
        .join(PrintJobORM::asset())
        .where_("queue = ?")
        .bind(queue.uid)
        .fetch_all(&mut *conn)
        .await
        .unwrap();
    RawHtml(Template::render(
        "pages/prints/queue",
        context! { queue, prints },
    ))
}

#[get("/")]
async fn index(pool: &State<PgPool>) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let queues = PrintQueueORM::select().fetch_all(&mut *conn).await.unwrap();

    RawHtml(Template::render("pages/prints/index", context! { queues }))
}

#[get("/history")]
pub async fn history() -> RawHtml<Template> {
    RawHtml(Template::render("pages/prints/history", context! {}))
}

#[get("/create?<device>&<asset>&<queue>")]
pub async fn create_page(
    guard: Guard,
    pool: &State<PgPool>,
    device: Option<Uuid>,
    asset: Option<Uuid>,
    queue: Option<Uuid>,
) -> RawHtml<Template> {
    let mut conn = pool.acquire().await.unwrap();
    let devices = DeviceORM::select().fetch_all(&mut *conn).await.unwrap();

    RawHtml(Template::render(
        "pages/prints/create",
        context! { devices, device, asset, queue },
    ))
}

#[post("/create", data = "<form>")]
pub async fn create(guard: Guard, pool: &State<PgPool>, form: Form<PrintForm>) -> Redirect {
    let print = form.into_inner();
    let mut conn = pool.acquire().await.unwrap();
    let device_id = print.device;
    let device = DeviceORM::select()
        .where_("uid = ?")
        .bind(device_id)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    let user = User::select()
        .where_("subject = ?")
        .bind(guard.claims.sub)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    let asset = AssetORM::select()
        .where_("uid = ?")
        .bind(print.asset.unwrap())
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    // if queue is missing, there should be an error indicated, or a default queue created I don't know which yet
    let queue = device.queue.unwrap();
    let job = print.into_print_job(user, asset, queue);
    job.insert(&mut *conn).await.unwrap();
    Redirect::to(format!("/devices/{}/queue", device.uid))
}

pub fn get_routes() -> Vec<Route> {
    routes![index, history, create_page, create, jobs]
}
