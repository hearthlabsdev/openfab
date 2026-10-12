use bambu_rs::client::LanMqttClient;
use bambu_rs::config::ResolvedTarget;
use bambu_rs::core::model::Model;

use openfab_drivers::capabilities::DriverCapabilities;
use openfab_drivers::config::{ConfigField, ConfigSchema, DriverMeta};
use openfab_drivers::errors::{DriverError, StatusErr};
use openfab_drivers::events::{EventType, Event};
use openfab_drivers::status::PrinterStatus;
use openfab_drivers::utils::get_value_type;
use openfab_drivers::{Callback, CallbackId, DeviceDriver};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncRead;
use tokio::task::JoinHandle;
use tokio::sync::broadcast::{Sender, Receiver, channel};
use std::time::Duration;
use bambu_rs::core::status::PrinterStatus as BambuStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BambuJobExtensions {
    pub gcode_state: Option<String>,
    pub print_error: Option<i64>,
    
    pub layer_num: Option<i64>,
    pub total_layer_num: Option<i64>,

    pub stg_cur: Option<i64>,
    pub stage: Option<String>,

    pub gcode_file: Option<String>,
    pub gcode_file_prepare_percent: Option<i64>,
    pub mc_print_stage: Option<i64>,
    pub mc_print_sub_stage: Option<i64>,
    pub mc_print_line_number: Option<i64>,
    pub print_type: Option<String>,
    pub stg: Vec<i64>,
    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BambuExtensions {
    
    pub error: Option<DeviceError>,
    
    /* used in base JobStatus fields
    pub mc_percent: Option<i64>,
    
    pub remaining_time_min: Option<i64>,
    */
    pub home_flag: Option<i64>,
    
    pub cooling_fan_speed: Option<i64>,
    pub spd_lvl: Option<i64>,
    pub subtask_name: Option<String>,
    pub filament: Option<Filament>,
    pub lights: Vec<LightReport>,
    pub ipcam: Option<Ipcam>,
    pub big_fan1_speed: Option<i64>,
    pub big_fan2_speed: Option<i64>,
    pub heatbreak_fan_speed: Option<i64>,
    pub fan_gear: Option<i64>,
    pub spd_mag: Option<i64>,
    
    pub nozzle_diameter: Option<String>,
    pub nozzle_type: Option<String>,
    pub sdcard: Option<bool>,
    pub ams_status: Option<i64>,
    pub ams_rfid_status: Option<i64>,
    pub hw_switch_state: Option<i64>,
    pub wifi_signal: Option<String>,
    pub task_id: Option<String>,
    pub subtask_id: Option<String>,
    pub project_id: Option<String>,
    pub profile_id: Option<String>,
    pub sequence_id: Option<String>,
    pub lifecycle: Option<String>,
    pub ams: Option<Ams>,
    pub upgrade: Option<Upgrade>,
    pub online: Option<Online>,
    pub upload: Option<Upload>,
    pub hms: Vec<HmsAlert>,
}

pub fn try_from_device_status(status: BambuStatus) -> Result<PrinterStatus, StatusErr> {
    Ok(PrinterStatus {
        bed_temp: status.bed_temper.ok_or(StatusErr::MissingBedTemp)?,
        target_bed_temp: status.bed_target,
        nozzle_temp: status.nozzle_temper.ok_or(StatusErr::MissingNozzleTemp)?,
        target_nozzle_temp: status.nozzle_target,
        chamber_temp: status.chamber_temper_raw,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientTarget {
    ip: String,
    access_code: String,
    model: String,
    serial: String,
    mqtt_port: u16,
    camera_port: u16,
    ftps_port: u16,
    detect_port: u16,
}

impl ClientTarget {
    pub fn resolve(self) -> ResolvedTarget {
        ResolvedTarget {
            ip: self.ip,
            access_code: self.access_code,
            model: Model::from_config_str(&self.model),
            serial: self.serial,
            mqtt_port: self.mqtt_port,
            camera_port: self.camera_port,
            ftps_port: self.ftps_port,
            detect_port: self.detect_port,
        }
    }
}

impl From<ClientTarget> for ResolvedTarget {
    fn from(target: ClientTarget) -> ResolvedTarget {
        target.resolve()
    }
}

pub struct Reporter {
    last: BambuStatus,
}

impl Reporter {
    pub fn new(last: BambuStatus) -> Self {
        Self { last }
    }
}

impl EventHandler for Reporter {
    type Status = BambuStatus;
    fn process(status: Self::Status) -> Vec<Event> {
        let mut events = Vec::new();
        if status != last {
            
        }
        events
    }
}

pub struct BambuDriver {
    // identified by serial number?
    clients: HashMap<String, BambuClient>,
}

pub struct BambuClient {
    client: Arc<LanMqttClient>,
    worker: JoinHandle<()>,
    sender: Sender<Event>,
    recv: Receiver<Event>,
    handler: Option<JoinHandle<Result<Option<PrinterStatus>, DriverError>>>,
}

impl BambuDriver {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn metadata() -> DriverMeta {
        DriverMeta {
            id: "bambu-lab-lan".into(),
            name: "Bambu Lab".into(),
            version: "0.1.0".into(),
            schema: ConfigSchema {
                fields: vec![
                    ConfigField {
                        name: "ip".to_string(),
                        field_type: "text".to_string(),
                        label: "IP Address or Hostname".to_string(),
                        default: None,
                        required: Some(true),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "access_code".to_string(),
                        field_type: "password".to_string(),
                        label: "access code".to_string(),
                        default: None,
                        required: Some(true),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "model".to_string(),
                        field_type: "text".to_string(),
                        label: "model".to_string(),
                        default: None,
                        required: Some(true),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "serial".to_string(),
                        field_type: "text".to_string(),
                        label: "Serial Number".to_string(),
                        default: None,
                        required: Some(true),
                        min: None,
                        max: None,
                    },
                    ConfigField {
                        name: "mqtt_port".to_string(),
                        field_type: "number".to_string(),
                        label: "MQTT Port".to_string(),
                        default: Some("8883".to_string()),
                        required: Some(true),
                        min: Some(1),
                        max: Some(65535),
                    },
                    ConfigField {
                        name: "ftps_port".to_string(),
                        field_type: "number".to_string(),
                        label: "FTPS Port".to_string(),
                        default: Some("21".to_string()),
                        required: Some(true),
                        min: Some(1),
                        max: Some(65535),
                    },
                    ConfigField {
                        name: "camera_port".to_string(),
                        field_type: "number".to_string(),
                        label: "Camera Port".to_string(),
                        default: Some("322".to_string()),
                        required: Some(true),
                        min: Some(1),
                        max: Some(65535),
                    },
                    ConfigField {
                        name: "detect_port".to_string(),
                        field_type: "number".to_string(),
                        label: "Detect Port".to_string(),
                        default: Some("1990".to_string()),
                        required: Some(true),
                        min: Some(1),
                        max: Some(65535),
                    },
                ],
            },
        }
    }
}

#[async_trait::async_trait]
impl DeviceDriver for BambuDriver {
    fn metadata(&self) -> DriverMeta {
        Self::metadata()
    }
    // Requires &self or &mut self to be callable on a `dyn DeviceDriver`
    async fn connect(&mut self, config: serde_json::Value) -> Result<(), DriverError> {
        let target: ClientTarget = serde_json::from_value(config)?;
        let serial = target.serial.clone();
        let resolved = target.resolve();
        let client = Arc::new(LanMqttClient::new(resolved));
        let worker_client = client.clone();
        let (sender, recv) = channel(20);
        let sender2 = sender.clone();

        match worker_client.fetch_async().await {
            Ok(event) => {
                let status = BambuStatus::from_state(event.get());
                sender2.send(Event::StatusUpdate(try_from_device_status(status).expect("failed to parse device status"))).expect("failed to send data");
            }
            Err(e) => {
                return Err(DriverError::Custom(e.to_string()));
            }
        }

        let handle = tokio::task::spawn(async move {
            loop {
                match worker_client.fetch_async().await {
                    Ok(event) => {
                        let status = BambuStatus::from_state(event.get());
                        sender2.send(Event::StatusUpdate(try_from_device_status(status).expect("failed to parse device status"))).expect("failed to send data");
                    }
                    Err(e) => {
                        /// I should update to have this properly logged both to standard out, and to database.
                        sender2.send(Event::Err(e.to_string())).expect("failed to send error");
                        continue;
                    }
                }

                tokio::time::sleep(Duration::from_millis(1000)).await
            }
        });
        let lan_client = BambuClient {
            worker: handle,
            client,
            sender,
            recv,
            handler: None,
        };
        println!("inserted with serial: {}", serial);
        self.clients.insert(serial, lan_client);
        
        Ok(())
    }
    async fn disconnect(&mut self, device: &str) -> Result<(), DriverError> {
        let device = self.clients.remove(device);
        match device {
            Some(device) => {
                device.worker.abort();
            }
            None => {}
        }
        Ok(())
    }
    async fn upload_file(&mut self) -> Result<(), DriverError> {
        unimplemented!();
    }
    async fn get_status(&self, device: &str) -> Result<PrinterStatus, DriverError> {
        unimplemented!();
    }

    fn subscribe(&self, device: &str) -> Result<Receiver<Event>, DriverError> {
        Ok(self.clients.get(device).ok_or(DriverError::DeviceNotFound(device.into()))?.sender.subscribe())
    }

    /// Register an event callback handler
    async fn register_handler(
        &mut self,
        device: &str,
        callback: Callback,
    ) -> Result<(), DriverError> {
        let client = self
            .clients
            .get_mut(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))?;
        let recv = client.sender.subscribe();
        let handle = tokio::spawn(async move {
            callback(recv).await
        });
        client.handler = Some(handle);
        Ok(())
    }
}
