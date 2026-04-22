use rocket::form::FromForm;

#[derive(Debug, FromForm)]
pub struct GroupForm {
    pub name: String,
    pub description: Option<String>,
}