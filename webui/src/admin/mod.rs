pub mod users;
pub mod groups;


pub fn get_routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    routes.extend(users::routes::get_routes());
    routes.extend(groups::routes::get_routes());
    routes
}