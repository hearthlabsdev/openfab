pub mod admin;
pub mod config;
pub mod dashboard;
pub mod discovery;
pub mod errors;
pub mod routes;
pub mod utils;

use crate::discovery::AdvertiseCmd;
use openfab::IppTxtRecords;
use openfab::advertise::{IppAdvertiser, IppBroadcastBuilder};

use rocket::tokio;
use tokio::sync::mpsc;

/// can this server handle livekit orchestration for per machine live viewing.
#[cfg(feature = "livekit")]
pub mod livekit;

use ormlite::Model;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
#[ormlite(table = "printers")]
pub struct PrinterORM {
    /// Stable internal identifier
    #[ormlite(primary_key)]
    pub uid: Uuid,

    /// Inventory item UID (optional but recommended)
    pub item: Option<Uuid>,

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

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
#[ormlite(table = "print_jobs")]
pub struct PrintJobORM {
    /// Stable job identifier
    #[ormlite(primary_key)]
    pub uid: Uuid,

    /// Owning user
    pub user: Uuid,

    /// Target queue (capability-based)
    pub queue: Uuid,

    /// Original filename or description
    pub title: String,

    /// MIME type (application/pdf, image/png, etc.)
    pub mime: String,

    /// Logical size (pages, layers, etc.)
    pub units: Option<f64>,

    /// Estimated cost (computed at submission)
    pub estimated_cost: Option<f64>,

    /// Job creation timestamp
    pub created_at: i64,

    /// Terminal lifecycle state
    pub state: String,

    /// When job entered terminal state
    pub completed_at: Option<i64>,
}

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
#[ormlite(table = "print_job_executions")]
pub struct PrintJobExecutionORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,

    /// Parent job
    pub job: Uuid,

    /// Assigned printer
    pub printer: Uuid,

    /// Attempt number (1, 2, 3...)
    pub attempt: i32,

    /// Execution state
    pub state: String,

    /// When execution started
    pub started_at: Option<i64>,

    /// When execution ended
    pub finished_at: Option<i64>,

    /// Failure reason (if any)
    pub error: Option<String>,
}

pub struct VirtualPrinter {}

impl VirtualPrinter {
    pub fn new() -> Self {
        VirtualPrinter {}
    }

    pub fn spawn_advertiser(&self) -> mpsc::Sender<AdvertiseCmd> {
        // Channel for sending advertise commands
        let (tx, mut rx) = mpsc::channel::<AdvertiseCmd>(16);

        // Spawn the advertiser task
        tokio::spawn(async move {
            let mut advertiser = match IppAdvertiser::new() {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Failed to start IPP advertiser: {e}");
                    return;
                }
            };

            // Example: advertise a default virtual printer on startup
            let default_txt = IppTxtRecords::new()
                .rp("ipp/print")
                .ty("Open Maker Virtual Printer")
                .product("(OpenMPD Virtual Printer)")
                .pdl("test/x.gcode,image/svg+xml,model/stl,model/obj")
                .note("Development / virtual printer");

            let _ = IppBroadcastBuilder::new("OpenMPD Printer", "openmpd.local.")
                .records(default_txt)
                .secure(false)
                .broadcast(&mut advertiser);

            // Event loop: react to new advertise commands
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    AdvertiseCmd::Advertise {
                        instance,
                        hostname,
                        txt,
                        secure,
                    } => {
                        if let Err(e) = IppBroadcastBuilder::new(&instance, &hostname)
                            .records(txt)
                            .secure(secure)
                            .broadcast(&mut advertiser)
                        {
                            eprintln!("Failed to advertise printer {instance}: {e}");
                        }
                    }
                }
            }

            // When rx closes, advertiser drops → services are unpublished
        });
        tx
    }
}
