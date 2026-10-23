use serverlib::admin::users::UserInfo;
use serverlib::admin::groups::GroupORM;
use serverlib::accounts::User;
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
use uuid::Uuid;

use serverlib::admin::ActivationCodeORM;
use serverlib::admin::permissions::*;
use serverlib::admin::users::UserActivationCode;

#[get("/users")]
pub async fn index(guard: Guard, pool: &State<PgPool>) -> Result<RawHtml<Template>, OpenFabErr> {
    let pool = guard.check_permission(READ_USERS, None).await?;

    let mut conn = pool.acquire().await?;
    let users = UserInfo::list_paginated(pool, 100, 0).await?;
    Ok(RawHtml(Template::render(
        "pages/admin/users/index",
        context! { users },
    )))
}

#[post("/users/create", data = "<form>")]
pub async fn create_user(guard: Guard, form: Form<UserForm<'_>>) -> Redirect {
    unimplemented!();
}

#[get("/activation-codes")]
pub async fn list_activation_codes(guard: Guard) -> Result<RawHtml<Template>, OpenFabErr> {
    let pool = guard.check_permission(READ_ACTIVATION_CODES, None).await?;
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

    form: Form<ActivationCodeForm>,
) -> Result<Redirect, OpenFabErr> {
    let pool = guard
        .check_permission(CREATE_ACTIVATION_CODES, None)
        .await?;
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

#[post("/activation-codes/<code>/delete")]
pub async fn delete_activation_code(guard: Guard, code: Uuid) -> Result<Redirect, OpenFabErr> {
    let pool = guard
        .check_permission(DELETE_ACTIVATION_CODES, None)
        .await?;
    ActivationCodeORM::delete_code(pool, code).await?;
    Ok(Redirect::to("/admin/activation-codes"))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        index,
        create_user,
        list_activation_codes,
        generate_activation_code,
        delete_activation_code,
    ]
}
