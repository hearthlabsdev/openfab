use crate::accounts::User;
use crate::accounts::{AccountConfig, LoginForm, RegisterForm};
use crate::utils::Guard;

use ormlite::Model;
use ormlite::postgres::PgPool;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use rocket::{Route, State, get, post, routes};
use rocket_dyn_templates::{Template, context};
use rocket_oidc::auth::AuthState;
use rocket_oidc::config::OIDCConfig;

#[get("/login")]
pub fn login_page(
    config: &State<AccountConfig>,
    oidc: &State<Vec<OIDCConfig>>,
) -> RawHtml<Template> {
    RawHtml(Template::render(
        "pages/accounts/login",
        context! { config: config.inner(), providers: oidc.inner() },
    ))
}

#[post("/login", data = "<login_form>")]
pub async fn login(
    jar: &CookieJar<'_>,
    auth: &State<AuthState>,
    pool: &State<PgPool>,
    login_form: Form<LoginForm>,
) -> Redirect {
    let login = login_form.into_inner();
    let mut conn = pool.acquire().await.unwrap();
    let user = User::select()
        .where_("email = ?")
        .bind(login.email)
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    if !crate::utils::verify_password(&login.password, &user.password.as_ref().unwrap()).unwrap() {
        panic!("invalid password");
    }

    auth.local_login(jar, "RS512", &user.email).await.unwrap();
    Redirect::to(format!("/"))
}

#[get("/register")]
pub fn register_page(config: &State<AccountConfig>) -> RawHtml<Template> {
    RawHtml(Template::render(
        "pages/accounts/register",
        context! { config: config.inner() },
    ))
}

#[post("/register", data = "<register_form>")]
pub async fn register(
    config: &State<AccountConfig>,
    pool: &State<PgPool>,
    register_form: Form<RegisterForm>,
) -> Redirect {
    if !config.allow_registration() {
        panic!("registration is disabled");
    }
    let mut conn = pool.acquire().await.unwrap();
    // Handle login logic here
    register_form
        .into_inner()
        .register()
        .unwrap()
        .insert(&mut *conn)
        .await
        .unwrap();
    Redirect::to(format!("/"))
}

#[get("/dashboard")]
pub async fn user_dashboard(guard: Guard) -> RawHtml<Template> {
    RawHtml(Template::render("pages/accounts/dashboard", context! {}))
}

pub fn get_routes() -> Vec<Route> {
    routes![login_page, login, register_page, register, user_dashboard]
}
