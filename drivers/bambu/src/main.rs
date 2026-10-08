use bambu_rs::client::StatusSource;
use bambu_rs::client::{ClientOptions, LanMqttClient};
use bambu_rs::config::ResolvedTarget;
use bambu_rs::core::{model::Model, status::PrinterStatus};

fn main() {
    println!("Hello, world!");
    let target = ResolvedTarget {
        ip: "192.168.30.166".into(),
        access_code: "e206c5fd".into(),
        model: Model::H2D,
        serial: "0948AD540900417".into(),
        mqtt_port: 8883,
        camera_port: 322,
        ftps_port: 21,
        detect_port: 1990,
    };

    let options = ClientOptions {
        max_outgoing: 2_000_000,
        max_incoming: 2_000_000,
        ..Default::default()
    };
    let client = LanMqttClient::with_options(target, options);
    let state = client.fetch_snapshot().unwrap();
    let st = PrinterStatus::from_state(state.get());

    eprintln!("state={:?}", st.state());
    eprintln!("status: {:?}", st);
}
