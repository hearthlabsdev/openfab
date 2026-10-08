pub mod apikeys;
pub mod groups;
pub mod permissions;
pub mod users;

use ormlite::Model;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "user_activaton_code")]
pub struct ActivationCodeORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
}

pub fn get_routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    routes.extend(users::ui::get_routes());
    routes.extend(groups::ui::get_routes());
    routes.extend(apikeys::get_routes());
    println!("routes: {:?}", routes);
    routes
}
