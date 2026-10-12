use openfab_drivers::capabilities::DriverCapabilities;
use openfab_drivers::config::{ConfigField, ConfigSchema, DriverMeta};
use openfab_drivers::errors::{DriverError, StatusErr};
use openfab_drivers::events::{Event, EventType};
use openfab_drivers::status::{PrinterStatus, JobStatus, State};
use openfab_drivers::utils::get_value_type;
use openfab_drivers::{Callback, CallbackId, DeviceDriver};
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::sync::broadcast::{Receiver, Sender, channel};
use tokio::task::JoinHandle;
use tokio::time::sleep;

// at some point it might be a good idea to setup a certificate validation chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DummyClientConfig {
    serial: String,
    target_nozzle_temp: f64,
    target_bed_temp: f64,
    increment: f64,
    poll_interval: i64,
}

pub struct DummyDriver {
    // identified by serial number?
    clients: HashMap<String, DummyClient>,
}

pub struct DummyState {
    pub bed_temp: f64,
    pub nozzle_temp: f64,
    pub time: i64,
    pub progress: i64,
    pub config: DummyClientConfig,
}

impl DummyState {
    pub fn new(config: DummyClientConfig) -> Self {
        Self {
            bed_temp: 0.0,
            nozzle_temp: 0.0,
            progress: 0,
            time: 0,
            config,
        }
    }
    pub fn increment(&mut self) {
        self.time += self.config.poll_interval;
        if self.bed_temp < self.config.target_bed_temp {
            self.bed_temp += self.config.increment;
        }
        if self.nozzle_temp < self.config.target_nozzle_temp {
            self.nozzle_temp += self.config.increment;
        }
        if self.progress < 100
            && self.bed_temp > self.config.target_bed_temp
            && self.nozzle_temp > self.config.target_nozzle_temp
        {
            self.progress = self.time / self.calculate_time_remaining();
        }
    }

    pub fn printer_status(&self) -> PrinterStatus {
        PrinterStatus::new(
            self.bed_temp,
            Some(self.config.target_bed_temp),
            self.nozzle_temp,
            Some(self.config.target_nozzle_temp),
            Some(self.bed_temp),
        )
    }

    pub fn calculate_time_remaining(&self) -> i64 {
        // 1. Calculate ticks/steps needed to reach target temperatures (the bottleneck)
        let bed_diff = (self.config.target_bed_temp - self.bed_temp).max(0.0);
        let nozzle_diff = (self.config.target_nozzle_temp - self.nozzle_temp).max(0.0);
        let max_diff = bed_diff.max(nozzle_diff);

        let heating_steps = if self.config.increment > 0.0 {
            (max_diff / self.config.increment).ceil() as i64
        } else {
            0
        };

        let heating_time = heating_steps * (self.config.poll_interval as i64);

        // 2. Mock a standard print duration after heating completes (e.g., 300 seconds total print time)
        let total_mock_print_duration = 300; 

        if heating_time > 0 {
            // Still heating up
            heating_time + total_mock_print_duration
        } else {
            // Heating complete, countdown the mock print duration
            let elapsed_print_time = self.time - heating_time;
            (total_mock_print_duration - elapsed_print_time).max(0)
        }
    }

    pub fn job_status(&self) -> JobStatus {
        JobStatus::new(self.calculate_time_remaining(), self.progress, State::Printing)
    }
}

pub struct DummyClient {
    worker: JoinHandle<()>,
    sender: Sender<Event>,
    // dropping the last receiver closest the channel
    recv: Receiver<Event>,
    handler: Option<JoinHandle<Result<Option<PrinterStatus>, DriverError>>>,
    state: Arc<Mutex<DummyState>>,
}

impl DummyDriver {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn metadata() -> DriverMeta {
        DriverMeta {
            id: "Dummylink".into(),
            name: "DummyLink Lan API".into(),
            version: "0.1.0".into(),
            schema: ConfigSchema {
                fields: vec![
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
                        name: "target_nozzle_temp".to_string(),
                        field_type: "number".to_string(),
                        label: "Target Nozzle Temperature".to_string(),
                        default: Some("300".into()),
                        required: Some(true),
                        min: Some(10),
                        max: Some(500),
                    },
                    ConfigField {
                        name: "increment".to_string(),
                        field_type: "number".to_string(),
                        label: "Increment".to_string(),
                        default: Some("4".into()),
                        required: Some(true),
                        min: Some(1),
                        max: Some(20),
                    },
                    ConfigField {
                        name: "target_bed_temp".to_string(),
                        field_type: "number".to_string(),
                        label: "Target Bed Temperature".to_string(),
                        default: Some("60".into()),
                        required: Some(false),
                        min: Some(1),
                        max: Some(500),
                    },
                    ConfigField {
                        name: "poll_interval".to_string(),
                        field_type: "number".to_string(),
                        label: "Poll Interval (ms)".to_string(),
                        default: Some("60".into()),
                        required: Some(false),
                        min: Some(1),
                        max: Some(5000),
                    },
                ],
            },
        }
    }
}

#[async_trait::async_trait]
impl DeviceDriver for DummyDriver {
    fn metadata(&self) -> DriverMeta {
        Self::metadata()
    }
    // Requires &self or &mut self to be callable on a `dyn DeviceDriver`
    async fn connect(&mut self, config: serde_json::Value) -> Result<(), DriverError> {
        let config: DummyClientConfig = serde_json::from_value(config)?;
        let serial = config.serial.clone();
        let poll_interval = config.poll_interval.clone();
        let state = Arc::new(Mutex::new(DummyState::new(config)));

        let (sender, recv) = tokio::sync::broadcast::channel(16);
        let sender2 = sender.clone();
        let state_mtx = state.clone();
        let handle = tokio::task::spawn(async move {
            loop {
                let mut state = state_mtx.lock().await;
                state.increment();
                let status = state.printer_status();
                let jbstat = state.job_status();
                sender2.send(Event::StatusUpdate(status)).expect("failed to send");
                sender2.send(Event::JobUpdate(jbstat)).expect("failed to send");
                sleep(Duration::from_millis(poll_interval as u64)).await;
            }
        });
        let lan_client = DummyClient {
            worker: handle,
            sender,
            recv,
            handler: None,
            state,
        };
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
        Ok(self.clients.get(device).ok_or(DriverError::DeviceNotFound(device.into()))?.state.lock().await.printer_status())
    }

    fn subscribe(&self, device: &str) -> Result<Receiver<Event>, DriverError> {
        Ok(self
            .clients
            .get(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))?
            .sender
            .subscribe())
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
