#![deny(unused)]
use openfab_drivers::config::{ConfigField, ConfigSchema, DriverMeta};
use openfab_drivers::errors::{DriverError, StatusErr};
use openfab_drivers::events::Event;
use openfab_drivers::files::FileEntry;
use openfab_drivers::status::{PrinterStatus, State as PrinterState};
use openfab_drivers::{BoxAsyncRead, DeviceDriver};
use prusa_rs::api::client::{ApiClient, ClientConfig};
use prusa_rs::models::status_printer::StatusPrinter;
use prusa_rs::models::status_printer::State as PrusaState;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast::{Receiver, Sender, channel};
use tokio::task::JoinHandle;
use tokio::time::sleep;

use approx::relative_eq;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PrinterStatusMsg {
    ok: Option<bool>,
    message: Option<String>,
}

impl PrinterStatusMsg {
    pub fn new(ok: Option<bool>, message: Option<String>) -> Self {
        Self { ok, message }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrusaExtensions {
    axis_x: Option<f64>,
    axis_y: Option<f64>,
    axis_z: Option<f64>,
    flow: Option<i32>,
    speed: Option<i32>,
    fan_hotend: Option<i32>,
    fan_print: Option<i32>,
    connection_status: Option<PrinterStatusMsg>,
    printer_status: Option<PrinterStatusMsg>,
}

impl PartialEq for PrusaExtensions {
    fn eq(&self, other: &Self) -> bool {
        // Helper closure or macro logic to compare Option<f64> with relative_eq
        let f64_eq = |a: &Option<f64>, b: &Option<f64>| match (a, b) {
            (Some(v1), Some(v2)) => relative_eq!(v1, v2, epsilon = 1.0_f64.to_radians()), // adjust epsilon as needed
            (None, None) => true,
            _ => false,
        };

        f64_eq(&self.axis_x, &other.axis_x)
            && f64_eq(&self.axis_y, &other.axis_y)
            && f64_eq(&self.axis_z, &other.axis_z)
            && self.flow == other.flow
            && self.speed == other.speed
            && self.fan_hotend == other.fan_hotend
            && self.fan_print == other.fan_print
            && self.connection_status == other.connection_status
            && self.printer_status == other.printer_status
    }
}

impl From<&StatusPrinter> for PrusaExtensions {
    fn from(status: &StatusPrinter) -> PrusaExtensions {
        PrusaExtensions {
            axis_x: status.axis_x,
            axis_y: status.axis_y,
            axis_z: status.axis_z,
            flow: status.flow,
            speed: status.speed,
            fan_hotend: status.fan_hotend,
            fan_print: status.fan_print,
            connection_status: status
                .status_connect
                .clone()
                .map(|v| PrinterStatusMsg::new(v.ok, v.message)),
            printer_status: status
                .status_printer
                .clone()
                .map(|v| PrinterStatusMsg::new(v.ok, v.message)),
        }
    }
}

pub fn match_state(state: PrusaState) -> PrinterState {
    match state {
        PrusaState::Idle => PrinterState::Idle,
        PrusaState::Busy => PrinterState::Busy,
        PrusaState::Printing => PrinterState::Printing,
        PrusaState::Paused => PrinterState::Paused,
        PrusaState::Finished => PrinterState::Finished,
        PrusaState::Stopped => PrinterState::Stopped,
        PrusaState::Error => PrinterState::Error,
        PrusaState::Attention => PrinterState::Attention,
        PrusaState::Ready => PrinterState::Ready,
    }
}

pub fn try_from_printer_status(status: StatusPrinter) -> Result<PrinterStatus, StatusErr> {
    Ok(PrinterStatus::with_extras(
        match_state(status.state),
        status.temp_bed.ok_or(StatusErr::MissingBedTemp)?,
        status.target_bed,
        status.temp_nozzle.ok_or(StatusErr::MissingNozzleTemp)?,
        status.target_nozzle,
        None,
        PrusaExtensions::from(&status),
    )?)
}

// at some point it might be a good idea to setup a certificate validation chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrusaDriverConfig {
    server: String,
    apikey: String,
    serial: String,
    poll_interval: u64,
    prefix: Option<String>,
    user_agent: Option<String>,
}

impl PrusaDriverConfig {
    pub fn client_config(&self) -> ClientConfig {
        ClientConfig {
            server: self.server.clone(),
            api_key: self.apikey.clone(),
            prefix: self.prefix.clone(),
            user_agent: self.user_agent.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub struct PrusaDriver {
    // identified by serial number?
    clients: HashMap<String, PrusaClient>,
}

#[derive(Debug)]
pub struct PrusaClient {
    client: Arc<ApiClient>,
    worker: JoinHandle<()>,
    sender: Sender<Event>,
    // dropping the last receiver closest the channel
    _recv: Receiver<Event>,
}

impl Drop for PrusaClient {
    fn drop(&mut self) {
        // 1. Signal the task to abort immediately, or let dropping `sender` close the channel
        self.worker.abort();

        // Note: If you must wait for the task to finish async cleanup before fully stopping,
        // you should trigger that in Rocket's `on_shutdown` fairing prior to `Drop` running.
    }
}

impl PrusaClient {
    pub(crate) async fn disconnect(&self) -> Result<(), DriverError> {
        self.worker.abort();
        Ok(())
    }
}

impl PrusaDriver {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn metadata() -> DriverMeta {
        DriverMeta {
            id: "prusalink".into(),
            name: "PrusaLink Lan API".into(),
            version: "0.1.0".into(),
            schema: ConfigSchema {
                fields: vec![
                    ConfigField {
                        name: "prefix".to_string(),
                        field_type: "text".to_string(),
                        label: "API Key Prefix".to_string(),
                        default: None,
                        required: Some(false),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "apikey".to_string(),
                        field_type: "password".to_string(),
                        label: "API Key".to_string(),
                        default: None,
                        required: Some(true),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "server".to_string(),
                        field_type: "text".to_string(),
                        label: "Local IP Address or Hostname".to_string(),
                        default: None,
                        required: Some(true),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "serial".to_string(),
                        field_type: "text".to_string(),
                        label: "Printer Serial Number".to_string(),
                        default: None,
                        required: Some(true),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "user_agent".to_string(),
                        field_type: "text".to_string(),
                        label: "Client User Agent".to_string(),
                        default: Some("openfab-prusa-driver".into()),
                        required: Some(false),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "poll_interval".to_string(),
                        field_type: "number".to_string(),
                        label: "Poll Interval (ms)".to_string(),
                        default: Some(100.to_string()),
                        required: None,
                        min: Some(1),
                        max: Some(60 * 1_000),
                    },
                ],
            },
        }
    }
}

#[async_trait::async_trait]
impl DeviceDriver for PrusaDriver {
    fn metadata(&self) -> DriverMeta {
        Self::metadata()
    }
    // Requires &self or &mut self to be callable on a `dyn DeviceDriver`
    async fn connect(&mut self, config: Value) -> Result<(), DriverError> {
        let config: PrusaDriverConfig = serde_json::from_value(config)?;
        let serial = config.serial.clone();
        let client_config = config.client_config();
        let client = Arc::new(ApiClient::new(client_config));
        let worker_client = client.clone();
        let poll_interval = config.poll_interval;
        let (sender, recv) = channel(16);
        let sender2 = sender.clone();
        let handle = tokio::task::spawn(async move {
            loop {
                match worker_client.get_status().await {
                    Ok(event) => {
                        let event = Event::StatusUpdate(
                            try_from_printer_status(*event.printer)
                                .expect("failed to get printer status"),
                        );
                        sender2.send(event).unwrap();
                        // I should run callbacks based on what has changed in the status but oh well
                    }
                    Err(e) => {
                        // I should update to have this properly logged both to standard out, and to database.
                        eprintln!("error occured in prusa driver: {}", e);
                        continue;
                    }
                }
                println!("poll interval: {}", poll_interval);
                sleep(Duration::from_millis(poll_interval)).await;
            }
        });
        let lan_client = PrusaClient {
            worker: handle,
            client,
            sender,
            _recv: recv,
        };
        self.clients.insert(serial, lan_client);
        Ok(())
    }

    async fn list_entries(&self, _device: &str, _dir: &str) -> Result<Vec<FileEntry>, DriverError> {
        Ok(Vec::new())
    }

    async fn disconnect(&mut self, device: &str) -> Result<(), DriverError> {
        let device = self.clients.remove(device);
        if let Some(device) = device {
            device.worker.abort();
        }
        Ok(())
    }
    async fn upload_file(
        &mut self,
        _device: &str,
        _path: &Path,
        _reader: BoxAsyncRead,
    ) -> Result<(), DriverError> {
        unimplemented!();
    }
    async fn shutdown(&self) -> Result<(), DriverError> {
        /*let keys = self.clients.iter().map(|(k, _)| k.to_string()).collect::<Vec<String>>();
        for key in keys.into_iter() {
            self.disconnect(&key).await?;
        }*/
        for (_, client) in self.clients.iter() {
            client.disconnect().await?;
        }
        Ok(())
    }
    async fn get_status(&self, device: &str) -> Result<PrinterStatus, DriverError> {
        let full = match self
            .clients
            .get(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))?
            .client
            .get_status()
            .await
        {
            Ok(status) => status,
            Err(e) => return Err(DriverError::Custom(e.to_string())),
        };
        Ok(try_from_printer_status(*full.printer)?)
    }

    fn subscribe(&self, device: &str) -> Result<Receiver<Event>, DriverError> {
        Ok(self
            .clients
            .get(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))?
            .sender
            .subscribe())
    }
}
