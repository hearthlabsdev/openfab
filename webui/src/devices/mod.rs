//! This module handles listing available printers based on discovery, manual add, and user permissions.
pub mod routes;
pub mod forms;

use uuid::Uuid;
use ormlite::Model;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
#[ormlite(table = "printers")]
pub struct PrinterORM {
    /// Stable internal identifier
    #[ormlite(primary_key)]
    pub uid: Uuid,

    /// Human-readable name (from IPP or admin override)
    pub name: String,

    /// mDNS instance name (used for correlation)
    pub mdns_instance: String,

    /// Hostname or IP at last discovery
    pub host: String,

    /// IPP port (usually 631, but not assumed)
    pub port: i32,

    /// When this printer was first observed
    pub first_seen: i64,

    /// Last successful discovery or IPP query
    pub last_seen: i64,

    /// Last successful IPP handshake
    pub last_verified: Option<i64>,

    /// Operational status
    pub status: String,

    /// Whether this printer is eligible for job dispatch
    pub enabled: bool,
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
