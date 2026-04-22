pub mod routes;
pub mod forms;

use uuid::Uuid;

pub struct GroupORM {
    pub uid: Uuid,
    pub name: String,
    pub description: Option<String>,
}