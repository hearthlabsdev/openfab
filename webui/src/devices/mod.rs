//! This module handles listing available printers based on discovery, manual add, and user permissions.
pub mod forms;
pub mod ui;

use openfab_drivers::native::NativeRuntime;
use openfab_drivers::runtime::DriverRuntime;
use openfab_drivers::utils::config_to_json;
use ormlite::Model;
use ormlite::postgres::PgConnection;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
pub use ui::get_routes;
use uuid::Uuid;

/*
#{ormlite(table = "print_jobs")}
pub struct DevicePrintsORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub user: Join<User>,
    pub queue: Uuid,
    pub asset: Join<AssetORM>,

}*/

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
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
    async fn list_configs(
        &self,
        conn: &mut PgConnection,
    ) -> Result<HashMap<String, String>, ormlite::Error> {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "device_configs")]
pub struct DeviceConfigORM {
    #[ormlite(primary_key)]
    uid: Uuid,
    device: Uuid,
    key: String,
    value: String,
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
