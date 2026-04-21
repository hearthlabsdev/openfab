pub mod routes;
use rocket::form::Form;
use rocket::FromForm;

#[derive(Debug, Clone, FromForm)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, FromForm)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
}