pub mod apikeys;
pub mod groups;
pub mod users;

pub fn get_routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();
    routes.extend(users::ui::get_routes());
    routes.extend(groups::ui::get_routes());
    routes.extend(apikeys::get_routes());
    println!("routes: {:?}", routes);
    routes
}
