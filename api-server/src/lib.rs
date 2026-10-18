pub mod devices;
pub mod errors;
pub mod library;
pub mod prints;
pub mod utils;

use rocket::{Build, Rocket};
use rocket_okapi::{
    openapi_get_routes,
    rapidoc::{GeneralConfig, HideShowConfig, RapiDocConfig, make_rapidoc},
    settings::UrlObject,
    swagger_ui::{SwaggerUIConfig, make_swagger_ui},
};

pub fn register_routes(r: Rocket<Build>) -> Rocket<Build> {
    r.mount(
        "/swagger-ui/",
        make_swagger_ui(&SwaggerUIConfig {
            urls: vec![
                UrlObject::new("OpenFab", "/api/openapi.json"),
                UrlObject::new("OpenFab Libraries", "/api/library/openapi.json"),
                UrlObject::new("OpenFab Devices", "/api/devices/openapi.json"),
                UrlObject::new("OpenFab Print Management", "/api/prints/openapi.json"),
            ],
            ..Default::default()
        }),
    )
    .mount(
        "/rapidoc/",
        make_rapidoc(&RapiDocConfig {
            general: GeneralConfig {
                spec_urls: vec![UrlObject::new("OpenFab", "/api/openapi.json")],
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
    .mount("/api/library/", library::get_routes())
    .mount("/api/devices", devices::get_routes())
    .mount("/api/prints", prints::get_routes())
    .mount(
        "/api/",
        openapi_get_routes![
            // print methods
        ],
    )
}
