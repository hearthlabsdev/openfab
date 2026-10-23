use openfab_sdk::Configuration;
use openfab_sdk::models::DeviceOrm as DeviceORM;
use async_compat::Compat;
slint::include_modules!();
use chrono::{DateTime, Local, TimeZone};
use slint::{ModelRc, VecModel};
use structopt::StructOpt;

// Helper function to convert epoch i64 to a formatted local datetime string
fn epoch_to_local_string(epoch: i64) -> String {
    // Note: If your epoch timestamps are in milliseconds instead of seconds, 
    // use DateTime::from_timestamp_millis(epoch) instead.
    DateTime::from_timestamp(epoch, 0)
        .map(|dt| {
            // Convert UTC datetime to local system time
            let local_dt: DateTime<Local> = dt.with_timezone(&Local);
            // Format how you want it to look, e.g., "2026-10-21 22:15:00"
            local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

pub fn convert_devices_to_slint(devices: &[DeviceORM]) -> ModelRc<OpenFABDevice> {
    let slint_devices: Vec<OpenFABDevice> = devices
        .iter()
        .map(|device| OpenFABDevice {
            uid: device.uid.to_string().into(),
            serial: device.serial.clone().into(),
            queue: device.queue.map_or_else(|| "".into(), |q| q.to_string().into()), // Fixed: device.queue instead of device
            name: device.name.clone().into(),
            
            // Convert epoch i64 timestamps to local datetime strings
            first_seen: epoch_to_local_string(device.first_seen).into(),
            last_seen: epoch_to_local_string(device.last_seen).into(),
            
            upload_dir: device.upload_dir.clone().unwrap_or_default().into(),
            status: device.status.clone().into(),
            enabled: device.enabled,
            driver_id: device.driver_id.clone().into(),
            driver_version: device.driver_version.clone().into(),
        })
        .collect();

    ModelRc::new(VecModel::from(slint_devices))
}

#[derive(Debug, Clone, StructOpt)]
pub struct Args {
    access_key: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("SLINT_BACKEND", "winit-software");
    }
    let args = Args::from_args();
    let app = AppView::new()?;

    let app_weak = app.as_weak();
    app.on_login(move |server, email, password| {
        println!("on login called with: {}, {}, {}", server, email, password);
        let app_weak = app_weak.clone();
        slint::spawn_local(Compat::new(async move {
            let login_result = Configuration::empty()
                .host(&server)
                .login(&email, &password)
                .await;

            match login_result {
                Ok(key) => {
                    let bearer_token = key.get_bearer().to_string();
                    println!("bearer: {}", bearer_token);

                    // 2. Upgrade the weak handle and update the UI properties
                    if let Some(app) = app_weak.upgrade() {
                        // Set the access token property (assumes 'access_token' is defined in your .slint)
                        app.set_access_token(bearer_token.clone().into());
                        app.set_current_page("devices".into());
                    }

                    let configuration = Configuration::new(key.get_bearer()).host(server).user_agent("OpenFAB Desktop");
                    match configuration.list_devices(None).await {
                        Ok(devices) => {
                            let slint_devices = convert_devices_to_slint(&devices);
                            // Convert your SDK structs into a Slint-compatible model (e.g., VecModel)
                            // let slint_devices = ...;

                            if let Some(app) = app_weak.upgrade() {
                                // 3. Populate the model property in Slint
                                app.set_access_token(bearer_token.into());
                                app.set_device_list(slint_devices.into());
                                app.set_current_page("devices".into());
                            }
                        }
                        Err(err) => {
                            eprintln!("Failed to fetch devices: {}", err);
                            if let Some(app) = app_weak.upgrade() {
                                app.set_current_page("login".into()); // Fallback on error
                            }
                        }
                    }
                    
                }
                Err(err) => {
                    eprintln!("Failed to sign in: {}", err);
                    // TODO: Handle login failure (e.g., set an error message property on UI)
                }
            }
        }));
    });
    app.run().expect("failed to run");
    Ok(())
}
