use crate::accounts::{RegisterForm, LoginForm, AccountConfig};
use crate::accounts::User;
use crate::utils::Guard;

use rocket_dyn_templates::{context, Template};
use rocket::response::content::RawHtml;
use rocket::{get, post, routes, State, Route};
use rocket::http::CookieJar;
use rocket::form::Form;
use rocket_oidc::auth::AuthState;
use rocket::response::Redirect;
use ormlite::postgres::PgPool;
use ormlite::Model;

#[get("/login")]
pub fn login_page(config: &State<AccountConfig>) -> RawHtml<Template> {
    RawHtml(Template::render("pages/accounts/login", context! { config: config.inner() }))
}

#[post("/login", data = "<login_form>")]
pub async fn login(jar: &CookieJar<'_>, auth: &State<AuthState>, pool: &State<PgPool>, login_form: Form<LoginForm>) -> Redirect {
    let login = login_form.into_inner();
    let mut conn = pool.acquire().await.unwrap();
    let user = User::select()
        .where_("email = ?")
        .bind(login.email)
        .fetch_one(&mut *conn)
        .await.unwrap();

    if !crate::utils::verify_password(&login.password, &user.password.as_ref().unwrap()).unwrap() {
        panic!("invalid password");
    }

    auth.local_login(jar, "RS512", &user.email.clone(), user.generate_local_user_id(None)).await.unwrap();
    Redirect::to(format!("/"))
}

#[get("/register")]
pub fn register_page(config: &State<AccountConfig>) -> RawHtml<Template> {
    RawHtml(Template::render("pages/accounts/register", context! { config: config.inner() }))
}

#[post("/register", data = "<register_form>")]
pub async fn register(config: &State<AccountConfig>, pool: &State<PgPool>, register_form: Form<RegisterForm>) -> Redirect {
    if !config.allow_registration() {
        panic!("registration is disabled");
    }
    let mut conn = pool.acquire().await.unwrap();
    // Handle login logic here
    register_form.into_inner().register().unwrap().insert(&mut *conn).await.unwrap();
    Redirect::to(format!("/"))
}

#[get("/dashboard")]
pub async fn user_dashboard(guard: Guard) -> RawHtml<Template> {
    RawHtml(Template::render("pages/accounts/dashboard", context!{}))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        login_page,
        login,
        register_page,
        register,
        user_dashboard
    ]
}