#![allow(unused)]
pub mod devices;
pub mod errors;
pub mod library;
pub mod prints;
pub mod utils;

use okapi::merge::marge_spec_list as merge_spec_list;
use rocket::{Build, Rocket};
use rocket::{get, serde::json::Json};
use rocket_okapi::{
    openapi_get_routes,
    rapidoc::{GeneralConfig, HideShowConfig, RapiDocConfig, make_rapidoc},
    settings::UrlObject,
    swagger_ui::{SwaggerUIConfig, make_swagger_ui},
};

#[get("/openapi-spec.json")]
pub fn spec_route() -> &'static str {
    include_str!("openapi-spec.json")
}

pub fn register_routes(r: Rocket<Build>) -> Rocket<Build> {
    let (library_routes, library_spec) = library::get_routes();
    let (device_routes, device_spec) = devices::get_routes();
    let (print_routes, print_spec) = prints::get_routes();

    let spec = merge_spec_list(&[
        ("/api/library", library_spec),
        ("/api/devices", device_spec),
        ("/api/prints", print_spec),
    ])
    .expect("failed to merge spec");
    let json = serde_json::to_string_pretty(&spec).unwrap();
    std::fs::write("openapi-spec.json", json).unwrap();
    r.mount(
        "/swagger-ui/",
        make_swagger_ui(&SwaggerUIConfig {
            urls: vec![UrlObject::new("OpenFab", "/api/openapi-spec.json")],
            ..Default::default()
        }),
    )
    .mount(
        "/rapidoc/",
        make_rapidoc(&RapiDocConfig {
            general: GeneralConfig {
                spec_urls: vec![UrlObject::new("OpenFab", "/api/openapi-spec.json")],
                ..Default::default()
            },
            hide_show: HideShowConfig {
                allow_spec_url_load: false,
                allow_spec_file_load: false,
                ..Default::default()
            },
            ..Default::default()
        }),
    )
    .mount("/api/library/", library_routes)
    .mount("/api/devices", device_routes)
    .mount("/api/prints", print_routes)
    .mount(
        "/api/",
        rocket::routes![
            spec_route,
            // print methods
        ],
    )
}
