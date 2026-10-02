pub mod forms;
pub mod ui;

use uuid::Uuid;

pub struct GroupORM {
    pub uid: Uuid,
    pub name: String,
    pub description: Option<String>,
}
