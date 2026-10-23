//! This module handles listing available printers based on discovery, manual add, and user permissions.

use crate::errors::{FileMatchErr, OpenFabErr};
use crate::library::{AssetORM, ObjectStore};
use crate::prints::PrintJob;
use crate::prints::{PrintJobORM, PrintQueueORM};
use crate::utils::{FileID, unix_epoch_seconds};
use openfab_drivers::files::FileEntry;
use openfab_drivers::native::NativeRuntime;
use openfab_drivers::runtime::DriverRuntime;
use openfab_drivers::status::{PrinterStatus, State};
use openfab_drivers::utils::config_to_json;
use ormlite::Model;
use ormlite::postgres::PgConnection;
use ormlite::postgres::PgPool;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use uuid::Uuid;

pub enum FileStatus {
    Conflict(FileEntry, DeviceFileORM),
    NotFound(DeviceFileORM),
    Matched(DeviceFileORM),
}

/// inorder to build out this table, files will have to be downloaded to be hashed.
#[derive(Debug, Clone, Serialize, Deserialize, Model, JsonSchema)]
#[ormlite(table = "device_files")]
pub struct DeviceFileORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    /// the device that this file is on
    pub device: Uuid,

    pub name: Option<String>,
    pub asset: Option<Uuid>,
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
    pub async fn get_file(pool: &PgPool, file: Uuid) -> Result<Self, ormlite::Error> {
        let mut conn = pool.acquire().await?;
        Ok(Self::select()
            .where_("uid = ?")
            .bind(file)
            .fetch_one(&mut *conn)
            .await?)
    }
    /// this is a shallow check
    pub fn check_against_device(&self, file: &FileEntry) -> Result<&Self, FileMatchErr> {
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

        Ok(self)
    }

    async fn from_asset<P: AsRef<Path>>(
        asset: &AssetORM,
        device: Uuid,
        upload_dir: P,
    ) -> Result<Self, OpenFabErr> {
        let mut upload_dir = upload_dir.as_ref().to_path_buf();
        upload_dir.push(&asset.name);
        let record = Self {
            uid: Uuid::new_v4(),
            device,
            asset: Some(asset.uid),
            name: Some(asset.name.to_string()),
            path: upload_dir.display().to_string(),
            hash: asset.hash.clone(),
            hash_alg: asset.hash_alg.clone(),
            modified: asset.modified(),
            size: asset.size,
            first_seen: unix_epoch_seconds(),
            uploaded: true,
        };
        Ok(record)
    }

    pub async fn check_upload_safety<R: DriverRuntime>(
        rt: &R,
        asset: &AssetORM,
        device: &DeviceORM,
        pool: &PgPool,
    ) -> Result<FileStatus, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        match Self::check_asset(rt, asset, device, &mut *conn).await {
            // file exists in the database, it exists on the device, and it matches the file on the device it's safe to upload
            Ok(record) => Ok(FileStatus::Matched(record)),
            // database record wasn't found, create it then rerun the check
            Err(FileMatchErr::RecordNotFound(_, _)) => {
                let record = Self::from_asset(
                    asset,
                    device.uid,
                    device
                        .upload_dir
                        .as_ref()
                        .map(|v| v.as_str())
                        .unwrap_or("/"),
                )
                .await?;
                match record.check_device(rt, device).await {
                    Ok(record) => Ok(FileStatus::Matched(record)),
                    Err(FileMatchErr::FileNotFound(record)) => Ok(FileStatus::NotFound(record)),
                    Err(val) => Err(OpenFabErr::FileMatchErr(val)),
                }
            }
            // file wasn't found on the device but was found in the database, it's safe to upload the file
            Err(FileMatchErr::FileNotFound(record)) => Ok(FileStatus::NotFound(record)),
            // the file exists in the database, and on the device but the file doesn't match records, it needs to be redownload, rehashed to verify
            Err(val) => Err(OpenFabErr::FileMatchErr(val)),
        }
    }

    /// attempts to grab the asset file from the database, then check it against the files on the physical device to ensure they haven't changed since last hash calculation.
    pub async fn check_asset<R: DriverRuntime>(
        rt: &R,
        asset: &AssetORM,
        device: &DeviceORM,
        conn: &mut PgConnection,
    ) -> Result<Self, FileMatchErr> {
        let file = Self::select()
            .where_("hash_alg = ? and hash = ?")
            .bind(&asset.hash_alg)
            .bind(&asset.hash)
            .fetch_one(&mut *conn)
            .await
            .map_err(|_| FileMatchErr::RecordNotFound(asset.name.clone(), device.name.clone()))?;

        Ok(file.check_device(rt, device).await?)
    }

    /// check the device to see if it has the matching file, if the file exists and it matches the database record
    /// this method returns a reference to the database object
    pub async fn check_device<R: DriverRuntime>(
        &self,
        rt: &R,
        device: &DeviceORM,
    ) -> Result<Self, FileMatchErr> {
        Ok(rt
            .list_entries(&device.driver_id, &device.serial, "/")
            .await
            .map_err(|_| FileMatchErr::DriverErr(device.driver_id.clone(), device.serial.clone()))?
            .into_iter()
            .filter(|v| self.path == v.path)
            .map(|v| Ok(self.check_against_device(&v)?.clone()))
            .next()
            .ok_or(FileMatchErr::FileNotFound(self.clone()))
            .flatten()?)
    }

    pub async fn list_files(device: Uuid, pool: &PgPool) -> Result<Vec<Self>, ormlite::Error> {
        let mut conn = pool.acquire().await?;
        Ok(Self::select()
            .where_("device = ?")
            .bind(device)
            .fetch_all(&mut *conn)
            .await?)
    }

    /// returns a placeholder so that the user can quickly view files while the reconciler does it's thing.
    fn placeholder(device: Uuid, entry: &FileEntry) -> Self {
        DeviceFileORM {
            uid: Uuid::new_v4(),
            device,
            asset: None,
            name: Some(entry.name.to_string()),
            path: entry.path.display().to_string(),
            hash: "".into(),
            hash_alg: "".into(),
            size: entry.size,
            modified: entry.modified,
            first_seen: unix_epoch_seconds(),
            uploaded: false,
        }
    }

    pub async fn reconcile<R: DriverRuntime>(
        rt: &R,
        device: Uuid,
        pool: &PgPool,
    ) -> Result<Vec<Self>, OpenFabErr> {
        // grab device to get driver_id, and serial number
        let mut conn = pool.acquire().await?;
        let device = DeviceORM::select()
            .where_("devices.uid = ?")
            .bind(device)
            .fetch_one(&mut *conn)
            .await?;

        // list files for the given device
        let files = rt
            .list_entries(&device.driver_id, &device.serial, "/")
            .await
            .expect("entry parsing failure")
            .into_iter()
            .map(|v| (v.path.display().to_string(), v))
            .collect::<HashMap<String, FileEntry>>();
        println!("files: {:?}", files);
        let entries = Self::list_files(device.uid, pool)
            .await?
            .into_iter()
            .map(|v| (v.path.clone(), v))
            .collect::<HashMap<String, DeviceFileORM>>();
        let mut results = Vec::with_capacity(files.len());

        for (key, val) in files.iter() {
            #[cfg(debug_assertions)]
            eprintln!("{key}:{:?}", val);
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
                            let placeholder = Self::placeholder(device.uid, &val);
                            results.push(placeholder);
                        }
                    }
                }
                None => {
                    // entry not found in the database, launch a download + hash
                    let placeholder = Self::placeholder(device.uid, &val);
                    let placeholder = placeholder.insert(&mut *conn).await?;
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
    pub library: Option<Uuid>,
    pub serial: String,
    pub queue: Option<Uuid>,
    /// Human-readable name (from IPP or admin override)
    pub name: String,

    /// When this printer was first observed
    pub first_seen: i64,

    /// Last successful discovery or IPP query
    pub last_seen: i64,

    /// upload directory where gcode is stored
    pub upload_dir: Option<String>,

    /// Operational status
    pub status: String,

    /// Whether this printer is eligible for job dispatch
    pub enabled: bool,

    pub driver_id: String,

    pub driver_version: String,
}

impl DeviceORM {
    pub async fn list_devices(pool: &PgPool) -> Result<Vec<Self>, OpenFabErr> {
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

    pub async fn get_device(pool: &PgPool, uid: Uuid) -> Result<Self, ormlite::Error> {
        let mut conn = pool.acquire().await?;
        let device = DeviceORM::select()
            .where_("devices.uid = ?")
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

    pub async fn get_by_queue(pool: &PgPool, queue: Uuid) -> Result<Self, ormlite::Error> {
        let mut conn = pool.acquire().await?;
        Ok(Self::select()
            .where_("queue = ?")
            .bind(queue)
            .fetch_one(&mut *conn)
            .await?)
    }

    pub async fn default_for_job(pool: &PgPool, job: Uuid) -> Result<Option<Self>, ormlite::Error> {
        let job = PrintJobORM::admin_get_print(pool, job).await?;
        let queue = PrintQueueORM::get_queue(pool, job.queue).await?;
        match Self::get_by_queue(pool, queue.uid).await {
            Ok(device) => Ok(Some(device)),
            _ => Ok(None),
        }
    }

    pub async fn get_device_queue(
        pool: &PgPool,
        device: Uuid,
    ) -> Result<Vec<PrintJob>, ormlite::Error> {
        let device = DeviceORM::get_device(pool, device).await?;
        let queue = match device.queue {
            Some(queue) => queue,
            None => return Ok(Vec::new()),
        };
        Ok(PrintQueueORM::list_prints(pool, queue).await?)
    }
    pub async fn upload_file<R: DriverRuntime>(
        &self,
        rt: &R,
        store: &ObjectStore,
        pool: &PgPool,
        asset: &AssetORM,
    ) -> Result<DeviceFileORM, OpenFabErr> {
        // make sure it is safe to upload the asset to the print
        // this also should return a status indicating if the file needs to be uploaded to the printer or not
        // fixed to use FileStatus
        let record = DeviceFileORM::check_upload_safety(rt, &asset, &self, pool)
            .await
            .expect("upload safety verification failure");
        // now the file needs to be uploaded
        let file = match record {
            FileStatus::Matched(record) => {
                // file is matched, print can begin as is
                #[cfg(debug_assertions)]
                eprintln!("file matched: {:?}", record);
                record
            }
            FileStatus::NotFound(record) => {
                // file needs to be uploaded to the device
                let data = asset.download_bytes(store).await?;
                rt.upload_bytes(&self.driver_id, &self.serial, &record.path, data)
                    .await?;
                record
            }
            FileStatus::Conflict(_, _) => {
                panic!("this shouldn't happen with how the code is written rn")
            }
        };
        Ok(file)
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
    state: String,
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
            state: status.state.to_string(),
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
            state: State::from_str(&status.state).expect("failed to parse state"),
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
