use bambu_rs::async_camera::CameraClient;
use bambu_rs::client::LanMqttClient;
use bambu_rs::config::ResolvedTarget;
use bambu_rs::core::model::Model;
use bambu_rs::core::status::PrinterStatus as BambuStatus;
use bambu_rs::core::status::{DeviceError, Filament, HmsAlert, Ipcam, LightReport};
use bambu_rs::async_ftp::FtpsClient;
use openfab_drivers::capabilities::DriverCapabilities;
use openfab_drivers::files::FileEntry;
use openfab_drivers::config::{ConfigField, ConfigSchema, DriverMeta};
use openfab_drivers::errors::{DriverError, JobStatusErr, PrintError, StatusErr};
use openfab_drivers::events::{CameraEvent, Event, EventHandler, EventType};
use openfab_drivers::status::{JobStatus, PrinterStatus, State};
use openfab_drivers::utils::get_value_type;
use openfab_drivers::{Callback, CallbackId, DeviceDriver};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::sync::broadcast::{Receiver, Sender, channel};
use tokio::sync::watch::{Receiver as Watcher, Sender as FrameSender, channel as camera_feed};
use tokio::task::JoinHandle;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BambuJobExtensions {
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

impl From<&BambuStatus> for BambuJobExtensions {
    fn from(status: &BambuStatus) -> BambuJobExtensions {
        BambuJobExtensions {
            print_error: status.print_error,
            layer_num: status.layer_num,
            total_layer_num: status.total_layer_num,
            stg_cur: status.stg_cur,
            stage: status.stage.clone(),
            gcode_file: status.gcode_file.clone(),
            gcode_file_prepare_percent: status.gcode_file_prepare_percent.clone(),
            mc_print_stage: status.mc_print_stage,
            mc_print_sub_stage: status.mc_print_sub_stage,
            mc_print_line_number: status.mc_print_line_number,
            print_type: status.print_type.clone(),
            stg: status.stg.clone(),
        }
    }
}

pub fn try_from_job_status(status: &BambuStatus) -> Result<JobStatus, JobStatusErr> {
    let state = match status.gcode_state.as_ref().map(|v| v.as_str()) {
        Some("IDLE") => State::Idle,
        Some("RUNNING") => State::Printing,
        Some("PAUSE") => State::Paused,
        Some("FINISH") => State::Finished,
        Some("FAILED") => State::Error,
        Some(val) => State::Unknown(val.to_string()),
        None => return Err(JobStatusErr::MissingJobState),
    };
    Ok(JobStatus::with_extras(
        status
            .remaining_time_min
            .ok_or(JobStatusErr::MissingTimeRemaining)?
            * 60,
        status.mc_percent.ok_or(JobStatusErr::MissingProgress)?,
        state,
        BambuJobExtensions::from(status),
    )?)
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

    // I may add this functionality in later, but not now
    //pub ams: Option<Ams>,
    //pub upgrade: Option<Upgrade>,
    //pub online: Option<Online>,
    //pub upload: Option<Upload>,
    pub hms: Vec<HmsAlert>,
}

impl From<BambuStatus> for BambuExtensions {
    fn from(status: BambuStatus) -> BambuExtensions {
        BambuExtensions {
            error: status.error,
            home_flag: status.home_flag,

            cooling_fan_speed: status.cooling_fan_speed,
            spd_lvl: status.spd_lvl,
            subtask_name: status.subtask_name,
            filament: status.filament,
            lights: status.lights,
            ipcam: status.ipcam,
            big_fan1_speed: status.big_fan1_speed,
            big_fan2_speed: status.big_fan2_speed,
            heatbreak_fan_speed: status.heatbreak_fan_speed,
            fan_gear: status.fan_gear,
            spd_mag: status.spd_mag,

            nozzle_diameter: status.nozzle_diameter,
            nozzle_type: status.nozzle_type,
            sdcard: status.sdcard,
            ams_status: status.ams_status,
            ams_rfid_status: status.ams_rfid_status,
            hw_switch_state: status.hw_switch_state,
            wifi_signal: status.wifi_signal,
            task_id: status.task_id,
            subtask_id: status.subtask_id,
            project_id: status.project_id,
            profile_id: status.profile_id,
            sequence_id: status.sequence_id,
            lifecycle: status.lifecycle,
            hms: status.hms,
        }
    }
}

pub fn try_from_device_status(status: BambuStatus) -> Result<PrinterStatus, StatusErr> {
    Ok(PrinterStatus::with_extras(
        status.bed_temper.ok_or(StatusErr::MissingBedTemp)?,
        status.bed_target,
        status.nozzle_temper.ok_or(StatusErr::MissingNozzleTemp)?,
        status.nozzle_target,
        status.chamber_temper_raw,
        BambuExtensions::from(status),
    )?)
}

pub fn file_entry_convert(v: String) -> openfab_drivers::files::FileEntry {
    //openfab_drivers::files::FileEntry::new(v.name, v.size, )
    FileEntry::new(v, PathBuf::new(), None)
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
    fn process(&mut self, status: Self::Status) -> Result<Vec<Event>, StatusErr> {
        let mut events = Vec::new();
        let last_job_status =
            try_from_job_status(&self.last).expect("failed to get last job status");

        let job_status = try_from_job_status(&status).expect("failed to get last job status");

        if job_status != last_job_status {
            let job_event = match job_status.get_event_type(&last_job_status) {
                EventType::JobUpdate => Event::JobUpdate(job_status),
                EventType::PrintStarted => Event::PrintStarted(job_status),
                EventType::PrintPaused => Event::PrintPaused(job_status),
                EventType::PrintResumed => Event::PrintResumed(job_status),
                EventType::PrintStopped => {
                    let err = match status.error {
                        Some(ref err) => Some(
                            PrintError::with_extras(
                                err.code.to_string(),
                                err.message.clone(),
                                Some(err.lookup_url.clone()),
                                ErrorExtras {
                                    hms: status.hms.clone(),
                                },
                            )
                            .expect("failed to serialize errors"),
                        ),
                        None => None,
                    };
                    Event::PrintStopped(job_status, err)
                }
                EventType::Err => match status.error {
                    Some(ref err) => {
                        PrintError::with_extras(
                            err.code.to_string(),
                            err.message.clone(),
                            Some(err.lookup_url.clone()),
                            ErrorExtras {
                                hms: status.hms.clone(),
                            },
                        )
                        .expect("failed to serialize errors");
                        Event::Err(serde_json::to_string(&err).expect("failed to serialize error"))
                    }
                    None => Event::Err(format!("unknown device error")),
                },
                _ => panic!("unsure how to handle"),
            };
            events.push(job_event);
        }
        let last_printer_status = try_from_device_status(self.last.clone())?;
        let printer_status = try_from_device_status(status.clone())?;
        if last_printer_status != printer_status {
            events.push(Event::StatusUpdate(printer_status));
        }
        self.last = status;
        Ok(events)
    }
}

pub struct BambuDriver {
    // identified by serial number?
    clients: HashMap<String, BambuClient>,
}

pub struct BambuCameraClient {
    handle: JoinHandle<()>,
    camera: FrameSender<CameraEvent>,
    watcher: Watcher<CameraEvent>,
}

impl BambuCameraClient {
    pub fn new(resolved: ResolvedTarget) -> Result<Self, DriverError> {
        let client = Arc::new(CameraClient::new(resolved));
        let (camera, watcher): (FrameSender<CameraEvent>, Watcher<CameraEvent>) =
            camera_feed(CameraEvent::Image(vec![]));
        let camera_sender = camera.clone();

        let handle = tokio::task::spawn(async move {
            loop {
                match client.snapshot().await {
                    Ok(event) => {
                        camera_sender
                            .send(CameraEvent::Image(event))
                            .expect("failed to send event");
                    }
                    Err(e) => {
                        camera_sender
                            .send(CameraEvent::Error(DriverError::Custom(e.to_string())))
                            .expect("failed to send error");
                        eprintln!("camera error: {e}");
                        continue;
                    }
                }

                tokio::time::sleep(Duration::from_millis(10)).await
            }
        });

        Ok(Self {
            camera,
            watcher,
            handle,
        })
    }
    pub async fn disconnect(self) {
        self.handle.abort();
    }

    pub fn subscribe(&self) -> Watcher<CameraEvent> {
        self.camera.subscribe()
    }
}

pub struct BambuClient {
    client: Arc<LanMqttClient>,
    worker: JoinHandle<()>,
    sender: Sender<Event>,
    camera: Option<BambuCameraClient>,
    files: FtpsClient,
    recv: Receiver<Event>,
    handler: Option<JoinHandle<Result<Option<PrinterStatus>, DriverError>>>,
}

impl BambuClient {
    pub async fn new(resolved: ResolvedTarget) -> Result<Self, DriverError> {
        let client = Arc::new(LanMqttClient::new(resolved.clone()));
        let worker_client = client.clone();
        let (sender, recv) = channel(20);
        let sender2 = sender.clone();
        let files = FtpsClient::new(resolved.clone());
        let last = match worker_client.fetch_async().await {
            Ok(event) => {
                let bambu_status = BambuStatus::from_state(event.get());
                let status = try_from_device_status(bambu_status.clone())
                    .expect("failed to parse device status");
                sender2
                    .send(Event::StatusUpdate(status))
                    .expect("failed to send data");
                bambu_status
            }
            Err(e) => {
                return Err(DriverError::Custom(e.to_string()));
            }
        };
        let mut reporter = Reporter::new(last);

        let handle = tokio::task::spawn(async move {
            loop {
                match worker_client.fetch_async().await {
                    Ok(event) => {
                        let events = reporter
                            .process(BambuStatus::from_state(event.get()))
                            .expect("failed to process event");
                        for event in events.into_iter() {
                            sender2.send(event).expect("failed to send event");
                        }
                    }
                    Err(e) => {
                        /// I should update to have this properly logged both to standard out, and to database.
                        sender2
                            .send(Event::Err(e.to_string()))
                            .expect("failed to send error");
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
            files,
            camera: None,
            handler: None,
        };
        Ok(lan_client)
    }
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
        let lan_client = BambuClient::new(resolved).await?;
        println!("inserted with serial: {}", serial);
        self.clients.insert(serial, lan_client);

        Ok(())
    }
    async fn disconnect(&mut self, device: &str) -> Result<(), DriverError> {
        let device = self.clients.remove(device);
        match device {
            Some(device) => {
                device.worker.abort();
                match device.camera {
                    Some(camera) => {
                        camera.disconnect().await;
                    }
                    None => {}
                };
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
        Ok(self
            .clients
            .get(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))?
            .sender
            .subscribe())
    }

    fn subscribe_camera(&self, device: &str) -> Result<Option<Watcher<CameraEvent>>, DriverError> {
        Ok(self
            .clients
            .get(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))?
            .camera
            .as_ref()
            .map(|v| v.subscribe()))
    }

    async fn list_entries(&self, device: &str, dir: &str) -> Result<Vec<FileEntry>, DriverError> {
        println!("in bambu driver runtime");
        let files = &self
            .clients
            .get(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))?
            .files;
        eprintln!("in list files after get files");
        Ok(
            files.list(dir)
            .await
            .map_err(|v| DriverError::Custom(v.to_string()))?
            .into_iter()
            .map(|v| file_entry_convert(v))
            .collect::<Vec<FileEntry>>())
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
        let handle = tokio::spawn(async move { callback(recv).await });
        client.handler = Some(handle);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorExtras {
    pub hms: Vec<HmsAlert>,
}
