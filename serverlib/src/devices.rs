//! This module handles listing available printers based on discovery, manual add, and user permissions.

use crate::errors::{FileMatchErr, OpenFabErr};
use crate::library::{AssetORM, ObjectStore};
use crate::utils::{FileID, unix_epoch_seconds};
use openfab_drivers::events::Event;
use openfab_drivers::files::FileEntry;
use openfab_drivers::native::NativeRuntime;
use openfab_drivers::runtime::DriverRuntime;
use openfab_drivers::status::PrinterStatus;
use openfab_drivers::utils::config_to_json;
use ormlite::Model;
use ormlite::postgres::PgConnection;
use ormlite::postgres::PgPool;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

/// inorder to build out this table, files will have to be downloaded to be hashed.
#[derive(Debug, Clone, Serialize, Deserialize, Model, JsonSchema)]
#[ormlite(table = "device_files")]
pub struct DeviceFileORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    /// the device that this file is on
    pub device: Uuid,

    /// path to the file on the printer
    pub path: String,

    /// content addressed cryptographic hash of the file contents.
    pub hash: String,

    /// algorithim used to generate the hash
    pub hash_alg: String,

    /// unix epoch of when the file was last modified
    pub modified: i64,

    /// the size of the file in bytes
    pub size: i64,

    /// when the file is seen on the printer storage, or when it is uploaded to the printer.
    pub first_seen: i64,

    /// was this file added using this codebase
    pub uploaded: bool,
}

impl DeviceFileORM {
    /// this is a shallow check
    pub fn check_against_device(&self, file: &FileEntry) -> Result<(), FileMatchErr> {
        if file.size != self.size {
            return Err(FileMatchErr::Size(self.size, file.size));
        }
        if file.modified != self.modified {
            return Err(FileMatchErr::Modified(self.modified, file.modified));
        }
        if file.path != self.path {
            return Err(FileMatchErr::Path(
                self.path.clone(),
                file.path.display().to_string(),
            ));
        }

        Ok(())
    }

    pub async fn list_files(
        device: Uuid,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, ormlite::Error> {
        Ok(Self::select()
            .where_("device = ?")
            .bind(device)
            .fetch_all(&mut *conn)
            .await?)
    }

    /// returns a placeholder so that the user can quickly view files while the reconciler does it's thing.
    fn placeholder(entry: &FileEntry) -> Self {
        unimplemented!();
    }

    pub async fn reconcile<R: DriverRuntime>(
        rt: &R,
        device: Uuid,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, OpenFabErr> {
        // grab device to get driver_id, and serial number
        let device = DeviceORM::select()
            .where_("uid = ?")
            .bind(device)
            .fetch_one(&mut *conn)
            .await?;

        // list files for the given device
        let files = rt
            .list_entries(&device.driver_id, &device.serial, "/")
            .await?
            .into_iter()
            .map(|v| (v.path.display().to_string(), v))
            .collect::<HashMap<String, FileEntry>>();

        let entries = Self::list_files(device.uid, conn)
            .await?
            .into_iter()
            .map(|v| (v.path.clone(), v))
            .collect::<HashMap<String, DeviceFileORM>>();
        let mut results = Vec::with_capacity(files.len());

        for (key, val) in files.iter() {
            match entries.get(key) {
                Some(entry) => {
                    // file is found in database, perform check
                    match entry.check_against_device(&val) {
                        // files match safe to continue
                        Ok(_) => results.push(entry.clone()),
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("files didn't match in shallow reconcilaton: {}", e);
                            // download and rehash asyncronously.
                            let placeholder = Self::placeholder(&val);
                            results.push(placeholder);
                        }
                    }
                }
                None => {
                    // entry not found in the database, launch a download + hash
                    let placeholder = Self::placeholder(&val);
                    results.push(placeholder);
                }
            }
        }
        Ok(results)
    }
}

impl FileID for DeviceFileORM {
    fn hash_alg(&self) -> &str {
        &self.hash_alg
    }
    fn hash(&self) -> &str {
        &self.hash
    }
    fn size(&self) -> i64 {
        self.size
    }
    fn modified(&self) -> i64 {
        self.modified
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Model, JsonSchema)]
#[ormlite(table = "device_models")]
pub struct DeviceModelORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,
    pub variants: Vec<String>,
    pub technology: Option<String>,
    pub family: Option<String>,
    pub bed_model: Option<Uuid>,
    pub bed_texture: Option<Uuid>,
    pub thumbnail: Option<Uuid>,
    pub default_materials: Vec<String>,
}

impl DeviceModelORM {
    pub async fn list(pool: &PgPool) -> Result<Vec<Self>, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        let models = DeviceModelORM::select().fetch_all(&mut *conn).await?;
        Ok(models)
    }
}

/*
#{ormlite(table = "print_jobs")}
pub struct DevicePrintsORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub user: Join<User>,
    pub queue: Uuid,
    pub asset: Join<AssetORM>,

}*/

#[derive(Debug, Model, Clone, Serialize, Deserialize, JsonSchema)]
#[ormlite(table = "devices")]
pub struct DeviceORM {
    /// Stable internal identifier
    #[ormlite(primary_key)]
    pub uid: Uuid,

    pub serial: String,
    pub queue: Option<Uuid>,
    /// Human-readable name (from IPP or admin override)
    pub name: String,

    /// When this printer was first observed
    pub first_seen: i64,

    /// Last successful discovery or IPP query
    pub last_seen: i64,

    /// Operational status
    pub status: String,

    /// Whether this printer is eligible for job dispatch
    pub enabled: bool,

    pub driver_id: String,

    pub driver_version: String,
}

impl DeviceORM {
    pub async fn list(pool: &PgPool) -> Result<Vec<Self>, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        let devices = DeviceORM::select().fetch_all(&mut *conn).await?;
        Ok(devices)
    }
    pub async fn list_configs(
        &self,
        conn: &mut PgConnection,
    ) -> Result<HashMap<String, String>, OpenFabErr> {
        let configs = DeviceConfigORM::select()
            .where_("device = ?")
            .bind(self.uid)
            .fetch_all(&mut *conn)
            .await?;

        Ok(configs
            .into_iter()
            .map(|config| (config.key, config.value))
            .collect())
    }

    pub async fn get_config(
        &self,
        rt: &NativeRuntime,
        conn: &mut PgConnection,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let driver_id = &self.driver_id;
        let schema = rt.config_schema(&driver_id).await?;
        let configs = self.list_configs(conn).await?;
        Ok(config_to_json(&schema, &configs)?)
    }

    pub async fn get_device(pool: &PgPool, uid: Uuid) -> Result<Self, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        let device = DeviceORM::select()
            .where_("uid = ?")
            .bind(uid)
            .fetch_one(&mut *conn)
            .await?;
        Ok(device)
    }

    pub async fn logs(pool: &PgPool, device_uid: Uuid) -> Result<Vec<DeviceLog>, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        let logs = DeviceLogORM::select()
            .where_("device = ?")
            .bind(device_uid)
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .map(|v| v.try_into())
            .collect::<Result<Vec<DeviceLog>, serde_json::Error>>()?;
        Ok(logs)
    }

    pub async fn start_print<R: DriverRuntime>(
        rt: &R,
        asset: &DeviceFileORM,
        store: &ObjectStore,
    ) -> Result<Receiver<Event>, OpenFabErr> {
        unimplemented!();
    }

    /// a wrapper around start_print that compares asset to see if a device file can be found.
    pub async fn start_asset_print<R: DriverRuntime>(
        rt: &R,
        asset: &AssetORM,
        store: &ObjectStore,
    ) -> Result<Receiver<Event>, OpenFabErr> {
        unimplemented!();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "device_configs")]
pub struct DeviceConfigORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub device: Uuid,
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
#[ormlite(table = "print_queues")]
pub struct PrintQueueORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,

    /// Stable route path (e.g. ipp/laser-cutter)
    pub rp: String,

    /// Display name
    pub name: String,

    /// Logical capability class
    pub capability: String,

    /// When this queue was created
    pub created_at: i64,

    /// Last time at least one device was associated
    pub last_active: Option<i64>,

    /// Whether the queue is advertised
    pub advertised: bool,
}

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
#[ormlite(table = "printer_queue_membership")]
pub struct PrinterQueueMembershipORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,

    pub printer: Uuid,
    pub queue: Uuid,

    /// Whether this printer is currently eligible
    pub active: bool,

    /// Last time this printer successfully handled a job
    pub last_job_at: Option<i64>,
}

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
#[ormlite(table = "printer_discovery_events")]
pub struct PrinterDiscoveryEventORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,

    pub mdns_instance: String,
    pub host: String,
    pub port: i32,

    pub discovered_at: i64,

    /// Raw TXT records (lossy but useful)
    pub txt: Option<String>,

    /// Whether resolution succeeded
    pub success: bool,
    /// error message to record if verification fails.
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Model, JsonSchema)]
#[ormlite(table = "device_logs")]
pub struct DeviceLogORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub device: Uuid,
    pub severity: String,
    pub date: i64,
    pub message: String,
    pub event: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeviceLog {
    pub uid: Uuid,
    pub device: Uuid,
    pub severity: String,
    pub date: i64,
    pub message: String,
    pub event: Option<DeviceStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeviceStatus {
    /// Current measured temperature of the heated bed in degrees Celsius.
    pub bed_temp: f64,
    /// Target temperature of the heated bed, if active.
    pub target_bed_temp: Option<f64>,
    /// Current measured temperature of the print head nozzle in degrees Celsius.
    pub nozzle_temp: f64,
    /// Target temperature of the print head nozzle, if active.
    pub target_nozzle_temp: Option<f64>,
    /// Current measured temperature of the print chamber (if equipped).
    pub chamber_temp: Option<f64>,
    /// Private catch-all for vendor-specific JSON fields, flattened during serialization.
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl From<PrinterStatus> for DeviceStatus {
    fn from(status: PrinterStatus) -> Self {
        Self {
            bed_temp: status.bed_temp,
            target_bed_temp: status.target_bed_temp,
            nozzle_temp: status.nozzle_temp,
            target_nozzle_temp: status.target_nozzle_temp,
            chamber_temp: status.chamber_temp,
            extra: status.extra,
        }
    }
}

impl From<DeviceStatus> for PrinterStatus {
    fn from(status: DeviceStatus) -> Self {
        Self {
            bed_temp: status.bed_temp,
            target_bed_temp: status.target_bed_temp,
            nozzle_temp: status.nozzle_temp,
            target_nozzle_temp: status.target_nozzle_temp,
            chamber_temp: status.chamber_temp,
            extra: status.extra,
        }
    }
}

impl TryFrom<DeviceLogORM> for DeviceLog {
    type Error = serde_json::Error;

    fn try_from(log: DeviceLogORM) -> Result<Self, Self::Error> {
        let event = log.event.and_then(|e| serde_json::from_str(&e).ok());
        Ok(Self {
            uid: log.uid,
            device: log.device,
            severity: log.severity,
            date: log.date,
            message: log.message,
            event,
        })
    }
}

impl DeviceLogORM {
    fn define(
        device: Uuid,
        severity: String,
        message: String,
        event: Option<PrinterStatus>,
    ) -> Self {
        Self {
            uid: Uuid::new_v4(),
            device,
            severity,
            date: unix_epoch_seconds(),
            message,
            event: event.map(|e| serde_json::to_string(&e).unwrap_or_else(|_| "unknown".into())),
        }
    }
    pub fn info(device: Uuid, event: PrinterStatus) -> Self {
        Self::define(device, "info".into(), "printer-status".into(), Some(event))
    }
    pub fn error(device: Uuid, message: String, event: Option<PrinterStatus>) -> Self {
        Self::define(device, "error".into(), message, event)
    }

    pub fn log(device: Uuid, message: String, event: Option<PrinterStatus>) -> Self {
        Self::define(device, "log".into(), message, event)
    }

    pub fn warn(device: Uuid, message: String, event: Option<PrinterStatus>) -> Self {
        Self::define(device, "warn".into(), message, event)
    }
}

#[test]
fn test_device_log_conversion() {
    let log_orm = DeviceLogORM {
        uid: Uuid::new_v4(),
        device: Uuid::new_v4(),
        severity: "info".into(),
        date: unix_epoch_seconds(),
        message: "printer-status".into(),
        event: Some(r#"{"bed_temp": 60.0, "target_bed_temp": 60.0, "nozzle_temp": 200.0, "target_nozzle_temp": 200.0}"#.into()),
    };

    let log: DeviceLog = log_orm.try_into().expect("Conversion should succeed");
    assert_eq!(log.severity, "info");
    assert_eq!(log.message, "printer-status");
    assert!(log.event.is_some());
}

#[test]
fn test_printer_status_device_status_serialization() {
    let status = PrinterStatus {
        bed_temp: 60.0,
        target_bed_temp: Some(60.0),
        nozzle_temp: 200.0,
        target_nozzle_temp: Some(200.0),
        chamber_temp: None,
        extra: HashMap::new(),
    };

    let serialized = serde_json::to_string(&status).expect("Serialization should succeed");
    let deserialized: DeviceStatus =
        serde_json::from_str(&serialized).expect("Deserialization should succeed");

    assert_eq!(status.bed_temp, deserialized.bed_temp);
    assert_eq!(status.target_bed_temp, deserialized.target_bed_temp);
    assert_eq!(status.nozzle_temp, deserialized.nozzle_temp);
    assert_eq!(status.target_nozzle_temp, deserialized.target_nozzle_temp);
    assert_eq!(status.chamber_temp, deserialized.chamber_temp);
}

#[test]
fn test_device_status_printer_status_conversion() {
    let device_status = DeviceStatus {
        bed_temp: 60.0,
        target_bed_temp: Some(60.0),
        nozzle_temp: 200.0,
        target_nozzle_temp: Some(200.0),
        chamber_temp: None,
        extra: HashMap::new(),
    };

    let printer_status: PrinterStatus = device_status.clone().into();
    let converted_back: DeviceStatus = printer_status.into();

    assert_eq!(device_status.bed_temp, converted_back.bed_temp);
    assert_eq!(
        device_status.target_bed_temp,
        converted_back.target_bed_temp
    );
    assert_eq!(device_status.nozzle_temp, converted_back.nozzle_temp);
    assert_eq!(
        device_status.target_nozzle_temp,
        converted_back.target_nozzle_temp
    );
    assert_eq!(device_status.chamber_temp, converted_back.chamber_temp);
}
