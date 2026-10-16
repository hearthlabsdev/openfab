use libunftp::ServerBuilder;
use openfab_drivers::capabilities::DriverCapabilities;
use openfab_drivers::config::{ConfigField, ConfigSchema, DriverMeta};
use openfab_drivers::errors::{DriverError, StatusErr};
use openfab_drivers::events::{Event, EventType};
use openfab_drivers::files::FileEntry;
use openfab_drivers::status::{JobStatus, PrinterStatus, State};
use openfab_drivers::utils::get_value_type;
use openfab_drivers::{Callback, CallbackId, DeviceDriver};
use std::net::IpAddr;
use suppaftp::tokio::AsyncRustlsConnector;
use tokio::sync::Mutex;
use tokio_rustls::TlsConnector as RustlsTlsConnnector;
use unftp_sbe_fs::Filesystem;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use suppaftp::tokio::AsyncRustlsFtpStream;
use tokio::io::AsyncRead;
use tokio::sync::broadcast::{Receiver, Sender, channel};
use tokio::task::JoinHandle;
use tokio::time::sleep;
//use thiserror::Error;

#[derive(Debug)]
struct AcceptSelfSigned(Arc<rustls::crypto::CryptoProvider>);

impl rustls::client::danger::ServerCertVerifier for AcceptSelfSigned {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls_pki_types::CertificateDer<'_>,
        _intermediates: &[rustls_pki_types::CertificateDer<'_>],
        _server_name: &rustls_pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls_pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls_pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        // Skipped on purpose: webpki cannot parse the v1 cert to extract the key.
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls_pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

/// A rustls [`ClientConfig`](rustls::ClientConfig) that accepts the printer's self-signed
/// v1 certificate. Built with the ring provider, matching the rest of the TLS stack.
pub fn lan_client_config() -> Result<Arc<rustls::ClientConfig>, rustls::Error> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptSelfSigned(provider)))
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/*
#[derive(Debug, Error)]
pub enum VirtualDriverError {
    #[error("FTPS Server Error: {0}")]
    ServerError(#[from] libunftp::ServerError),
}*/

pub struct FTPSServer {
    path: PathBuf,
    addr: IpAddr,
    port: u16,
    handle: tokio::task::JoinHandle<()>,
}

impl FTPSServer {
    pub async fn new<P: AsRef<Path>>(
        path: P,
        addr: IpAddr,
        port: u16,
    ) -> Result<Self, DriverError> {
        let ftp_home = path.as_ref().to_path_buf();
        let home = ftp_home.clone();
        let server = ServerBuilder::new(Box::new(move || {
            Filesystem::new(ftp_home.clone()).expect("failed to create filesystem")
        }))
        .ftps(
            "/home/cardinal/projects/hearthlabs/openfab/keys/backend.cert.pem",
            "/home/cardinal/projects/hearthlabs/openfab/keys/backend.key.pem",
        )
        .passive_ports(50000..=65535)
        .build()
        .map_err(|e| DriverError::Custom(e.to_string()))?;
        let handle = tokio::task::spawn(async move {
            server.listen("127.0.0.1:2121").await;
        });
        Ok(Self {
            path: home,
            addr,
            port,
            handle,
        })
    }
    pub async fn default() -> Result<Self, DriverError> {
        let home = std::env::temp_dir();
        let addr = IpAddr::V4([127, 0, 0, 1].into());
        Self::new(home, addr, 2121).await
    }
    pub fn close(&self) {
        self.handle.abort()
    }

    pub async fn connect(&self, domain: &str) -> Result<AsyncRustlsFtpStream, DriverError> {

        let mut stream =
            AsyncRustlsFtpStream::connect(format!("{}:{}", self.addr, self.port))
                .await
                .map_err(|e| DriverError::Custom(e.to_string()))?
                .into_secure(
                    AsyncRustlsConnector::from(RustlsTlsConnnector::from(
                        lan_client_config().map_err(|e| DriverError::Custom(e.to_string()))?,
                    )),
                    &domain,
                )
                .await
                .map_err(|e| DriverError::Custom(e.to_string()))?;
        stream
            .login("", "")
            .await
            .map_err(|e| DriverError::Custom(e.to_string()))?;
        Ok(stream)
    }
}

// at some point it might be a good idea to setup a certificate validation chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualClientConfig {
    serial: String,
    target_nozzle_temp: f64,
    target_bed_temp: f64,
    increment: f64,
    poll_interval: i64,
}

pub struct VirtualDriver {
    // identified by serial number?
    clients: HashMap<String, VirtualClient>,
    server: FTPSServer,
}

pub struct VirtualState {
    pub bed_temp: f64,
    pub nozzle_temp: f64,
    pub time: i64,
    pub progress: i64,
    pub config: VirtualClientConfig,
}

impl VirtualState {
    pub fn new(config: VirtualClientConfig) -> Self {
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
        if self.progress < 100 {
            self.progress += 1; //self.time / self.calculate_time_remaining();
        }
    }

    pub fn reset(&mut self) {
        self.bed_temp = 0.0;
        self.nozzle_temp = 0.0;
        self.time = 0;
        self.progress = 0;
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
        let status = JobStatus::new(
            self.calculate_time_remaining(),
            self.progress,
            State::Printing,
        );
        status
    }
}

pub struct VirtualClient {
    worker: JoinHandle<()>,
    sender: Sender<Event>,
    // dropping the last receiver closest the channel
    recv: Receiver<Event>,
    handler: Option<JoinHandle<Result<Option<PrinterStatus>, DriverError>>>,
    state: Arc<Mutex<VirtualState>>,
    ftps: Arc<Mutex<AsyncRustlsFtpStream>>,
}

impl VirtualDriver {
    pub async fn new() -> Result<Self, DriverError> {
        Ok(Self {
            clients: HashMap::new(),
            server: FTPSServer::default().await?,
        })
    }

    pub fn metadata() -> DriverMeta {
        DriverMeta {
            id: "Virtuallink".into(),
            name: "VirtualLink Lan API".into(),
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
impl DeviceDriver for VirtualDriver {
    fn metadata(&self) -> DriverMeta {
        Self::metadata()
    }
    // Requires &self or &mut self to be callable on a `dyn DeviceDriver`
    async fn connect(&mut self, config: serde_json::Value) -> Result<(), DriverError> {
        let config: VirtualClientConfig = serde_json::from_value(config)?;
        let serial = config.serial.clone();
        let poll_interval = config.poll_interval.clone();
        let state = Arc::new(Mutex::new(VirtualState::new(config)));

        let (sender, recv) = tokio::sync::broadcast::channel(16);
        let sender2 = sender.clone();
        let state_mtx = state.clone();
        let domain = "localhost";
        let mut stream = self.server.connect("localhost").await?;
        
        let ftps = Arc::new(Mutex::new(stream));
        let handle = tokio::task::spawn(async move {
            loop {
                let mut state = state_mtx.lock().await;
                state.increment();
                let status = state.printer_status();
                let jbstat = state.job_status();
                sender2
                    .send(Event::StatusUpdate(status))
                    .expect("failed to send");
                sender2
                    .send(Event::JobUpdate(jbstat))
                    .expect("failed to send");
                sleep(Duration::from_millis(poll_interval as u64)).await;
            }
        });
        let lan_client = VirtualClient {
            worker: handle,
            sender,
            recv,
            handler: None,
            state,
            ftps,
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
        Ok(self
            .clients
            .get(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))?
            .state
            .lock()
            .await
            .printer_status())
    }

    fn subscribe(&self, device: &str) -> Result<Receiver<Event>, DriverError> {
        Ok(self
            .clients
            .get(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))?
            .sender
            .subscribe())
    }

    async fn list_entries(&self, device: &str, dir: &str) -> Result<Vec<FileEntry>, DriverError> {
        Ok(self
            .clients
            .get(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))?
            .ftps
            .lock()
            .await
            .list(Some(dir))
            .await
            .map_err(|e| DriverError::Custom(e.to_string()))?
            .into_iter()
            .map(|v| FileEntry::new(&v, PathBuf::new(), None))
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
