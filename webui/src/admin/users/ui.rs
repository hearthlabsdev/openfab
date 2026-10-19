use crate::accounts::User;
use crate::admin::users::forms::ActivationCodeForm;
use crate::admin::users::forms::UserForm;
use crate::errors::OpenFabErr;
use crate::utils::Guard;
use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use rocket::{Route, State, get, post, routes};
use rocket_dyn_templates::{Template, context};

use serverlib::admin::ActivationCodeORM;
use serverlib::admin::users::UserActivationCode;

#[get("/users")]
pub async fn index(guard: Guard, pool: &State<PgPool>) -> Result<RawHtml<Template>, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let users = User::select().fetch_all(&mut *conn).await?;

    Ok(RawHtml(Template::render(
        "pages/admin/users/index",
        context! { users },
    )))
}

#[post("/users/create", data = "<form>")]
pub async fn create_user(form: Form<UserForm<'_>>) -> Redirect {
    unimplemented!();
}

#[get("/activation-codes")]
pub async fn list_activation_codes(
    guard: Guard,
    pool: &State<PgPool>,
) -> Result<RawHtml<Template>, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let codes = ActivationCodeORM::select()
        .join(ActivationCodeORM::account())
        .join(ActivationCodeORM::creator())
        .fetch_all(&mut *conn)
        .await?;

    Ok(RawHtml(Template::render(
        "pages/admin/activation",
        context! { codes },
    )))
}

#[post("/activation-codes", data = "<form>")]
pub async fn generate_activation_code(
    guard: Guard,
    pool: &State<PgPool>,
    form: Form<ActivationCodeForm>,
) -> Result<Redirect, OpenFabErr> {
    let mut conn = pool.acquire().await?;

    let creator = User::select()
        .where_("subject = ?")
        .bind(guard.claims.sub)
        .fetch_one(&mut *conn)
        .await?;

    let form = form.into_inner();
    let code = UserActivationCode::generate_for_user(creator.uid(), form.user, form.ttl);
    code.insert(&mut *conn).await?;
    Ok(Redirect::to("/admin/activation-codes"))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        index,
        create_user,
        list_activation_codes,
        generate_activation_code
    ]
}
