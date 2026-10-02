pub mod devices;
pub mod library;
pub mod prints;

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
            urls: vec![UrlObject::new("OpenFab", "/api/openapi.json")],
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
    .mount(
        "/api/",
        openapi_get_routes![
            // library methods
            library::list_assets,
            library::get_asset,
            library::create_asset,
            library::update_asset,
            library::delete_asset,
            // device methods
            devices::list_devices,
            devices::get_device,
            devices::create_device,
            devices::update_device,
            devices::delete_device,
            // print methods
            prints::list_prints,
            prints::get_print,
            prints::create_print,
            prints::get_print_status,
            prints::pause_print,
            prints::stop_print,
            prints::delete_print,
        ],
    )
}
