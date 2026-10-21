#![deny(unused)]
use libunftp::ServerBuilder;
use openfab_drivers::config::{ConfigField, ConfigSchema, DriverMeta};
use openfab_drivers::errors::DriverError;
use openfab_drivers::events::Event;
use openfab_drivers::files::FileEntry;
use openfab_drivers::status::{JobStatus, PrinterStatus, State as PrinterState};
use openfab_drivers::{BoxAsyncRead, DeviceDriver};
use std::net::IpAddr;
use suppaftp::tokio::AsyncRustlsConnector;
//use suppaftp::types::FtpError;
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
    handle: tokio::task::JoinHandle<()>,
}

impl FTPSServer {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self, DriverError> {
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
            server
                .listen("127.0.0.1:2121")
                .await
                .expect("failed to run server");
        });
        Ok(Self { path: home, handle })
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub async fn default() -> Result<Self, DriverError> {
        let home = std::env::temp_dir();
        //let addr = IpAddr::V4([127, 0, 0, 1].into());
        Self::new(home).await
    }
    pub fn close(&self) {
        self.handle.abort()
    }

    /*pub async fn connect(&self, domain: &str) -> Result<FTPSClient, DriverError> {
        Ok(FTPSClient::new(self.server.addr, self.server.port, domain))
    }*/
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FTPSClient {
    addr: IpAddr,
    port: u16,
    domain: String,
}

impl Default for FTPSClient {
    fn default() -> FTPSClient {
        Self {
            addr: "127.0.0.1".parse().expect("failed to parse IP addr"),
            port: 2121,
            domain: "localhost".into(),
        }
    }
}

impl FTPSClient {
    pub(crate) async fn connect(&self) -> Result<AsyncRustlsFtpStream, DriverError> {
        let mut stream = AsyncRustlsFtpStream::connect(format!("{}:{}", self.addr, self.port))
            .await
            .map_err(|e| DriverError::Custom(e.to_string()))?
            .into_secure(
                AsyncRustlsConnector::from(RustlsTlsConnnector::from(
                    lan_client_config().map_err(|e| DriverError::Custom(e.to_string()))?,
                )),
                &self.domain,
            )
            .await
            .map_err(|e| DriverError::Custom(e.to_string()))?;
        stream
            .login("", "")
            .await
            .map_err(|e| DriverError::Custom(e.to_string()))?;
        stream
            .transfer_type(suppaftp::types::FileType::Binary)
            .await
            .map_err(|e| DriverError::Custom(e.to_string()))?;
        Ok(stream)
    }

    pub fn from_config(config: &VirtualClientConfig) -> Self {
        Self {
            addr: config.ftps_addr.unwrap_or(
                "127.0.0.1"
                    .parse()
                    .expect("failed to parse link local addr"),
            ),
            port: config.ftps_port.unwrap_or(2121),
            domain: config.ftps_domain.clone().unwrap_or("localhost".into()),
        }
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
    ftps_addr: Option<IpAddr>,
    ftps_port: Option<u16>,
    ftps_domain: Option<String>,
}

pub struct VirtualDriver {
    // identified by serial number?
    clients: HashMap<String, VirtualClient>,
    server: FTPSServer,
}

pub struct VirtualState {
    pub paused: bool,
    pub state: PrinterState,
    pub bed_temp: f64,
    pub nozzle_temp: f64,
    pub time: i64,
    pub progress: i64,
    pub config: VirtualClientConfig,
}

impl VirtualState {
    pub fn new(config: VirtualClientConfig) -> Self {
        Self {
            paused: false,
            state: PrinterState::Idle,
            bed_temp: 0.0,
            nozzle_temp: 0.0,
            progress: 0,
            time: 0,
            config,
        }
    }
    pub fn increment(&mut self) {
        self.state = PrinterState::Printing;
        if self.paused {
            return;
        }
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
        self.state = PrinterState::Idle;
    }

    pub fn start_print(&mut self) {
        self.state = PrinterState::Printing;
    }

    pub fn pause_print(&mut self) {
        self.paused = true;
    }

    pub fn resume_print(&mut self) {
        self.paused = false;
    }

    pub fn stop_print(&mut self) {
        self.reset();
    }

    pub fn printer_status(&self) -> PrinterStatus {
        PrinterStatus::new(
            self.state.clone(),
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

        let heating_time = heating_steps * (self.config.poll_interval);

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
        JobStatus::new(self.calculate_time_remaining(), self.progress)
    }
}

pub struct VirtualClient {
    worker: JoinHandle<()>,
    sender: Sender<Event>,
    // dropping the last receiver closest the channel
    _recv: Receiver<Event>,
    state: Arc<Mutex<VirtualState>>,
    ftps: FTPSClient,
}

impl Drop for VirtualClient {
    fn drop(&mut self) {
        // 1. Signal the task to abort immediately, or let dropping `sender` close the channel
        self.worker.abort();

        // Note: If you must wait for the task to finish async cleanup before fully stopping,
        // you should trigger that in Rocket's `on_shutdown` fairing prior to `Drop` running.
    }
}

impl VirtualClient {
    pub(crate) async fn disconnect(&self) -> Result<(), DriverError> {
        self.worker.abort();
        Ok(())
    }

    pub(crate) async fn upload_file<R: AsyncRead + Unpin, P: AsRef<Path>>(
        &self,
        path: P,
        mut reader: R,
    ) -> Result<u64, DriverError> {
        let path = path.as_ref().display().to_string();
        Ok(self
            .ftps
            .connect()
            .await?
            .put_file(path, &mut reader)
            .await
            .map_err(|e| DriverError::Custom(e.to_string()))?)
    }
}

impl VirtualDriver {
    pub async fn new() -> Result<Self, DriverError> {
        Ok(Self {
            clients: HashMap::new(),
            server: FTPSServer::default().await?,
        })
    }

    pub fn get_device(&self, device: &str) -> Result<&VirtualClient, DriverError> {
        self.clients
            .get(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))
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
    async fn connect(&mut self, config: Value) -> Result<(), DriverError> {
        let config: VirtualClientConfig = serde_json::from_value(config)?;
        let ftps = FTPSClient::from_config(&config);
        let serial = config.serial.clone();
        let poll_interval = config.poll_interval;
        let state = Arc::new(Mutex::new(VirtualState::new(config)));

        let (sender, recv) = channel(16);
        let sender2 = sender.clone();
        let state_mtx = state.clone();

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
            _recv: recv,
            state,
            ftps,
        };
        self.clients.insert(serial, lan_client);
        Ok(())
    }
    async fn disconnect(&mut self, device: &str) -> Result<(), DriverError> {
        let device = self.clients.remove(device);
        if let Some(device) = device {
            device.disconnect().await?;
        }
        Ok(())
    }
    async fn upload_file(
        &mut self,
        device: &str,
        path: &Path,
        reader: BoxAsyncRead,
    ) -> Result<(), DriverError> {
        self.clients
            .get(device)
            .ok_or(DriverError::DeviceNotFound(device.into()))?
            .upload_file(path, reader)
            .await?;
        Ok(())
    }
    async fn shutdown(&self) -> Result<(), DriverError> {
        /*let keys = self.clients.iter().map(|(k, _)| k.to_string()).collect::<Vec<String>>();
        for key in keys.into_iter() {
            self.disconnect(&key).await?;
        }*/
        for (_, client) in self.clients.iter() {
            client.disconnect().await?;
        }
        self.server.handle.abort();
        Ok(())
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
            .connect()
            .await?
            .mlsd(Some(dir))
            .await
            .map_err(|e| DriverError::Custom(e.to_string()))?
            .into_iter()
            .map(|v| {
                let file = suppaftp::list::ListParser::parse_mlsd(&v)
                    .map_err(|e| DriverError::Custom(e.to_string()))?;
                let mut path = PathBuf::new();
                let ts: i64 = file
                    .modified()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|_| DriverError::Custom(format!("failed to get unix epoch")))?
                    .as_secs()
                    .try_into()
                    .map_err(|_| DriverError::Custom(format!("failed to convert to i64")))?;

                path.push(dir);
                path.push(v.clone());
                Ok(FileEntry::new(file.name(), path, file.size() as i64, ts))
            })
            .collect::<Result<Vec<FileEntry>, DriverError>>()?)
    }

    async fn start_print(&self, device: &str, _path: &Path) -> Result<(), DriverError> {
        self.get_device(device)?.state.lock().await.start_print();
        Ok(())
    }
    async fn pause_print(&self, device: &str) -> Result<(), DriverError> {
        self.get_device(device)?.state.lock().await.pause_print();
        Ok(())
    }

    async fn resume_print(&self, device: &str) -> Result<(), DriverError> {
        self.get_device(device)?.state.lock().await.resume_print();
        Ok(())
    }
    async fn stop_print(&self, device: &str) -> Result<(), DriverError> {
        self.get_device(device)?.state.lock().await.stop_print();
        Ok(())
    }
}
