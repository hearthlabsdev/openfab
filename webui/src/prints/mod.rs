pub mod forms;
pub mod ui;
pub use ui::get_routes;

use ormlite::Model;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::unix_epoch_seconds;
use crate::accounts::User;
use crate::library::AssetORM;
use ormlite::model::Join;
use ormlite::model::JoinMeta;

#[derive(Debug, Model, Serialize, Deserialize)]
#[ormlite(table = "print_jobs")]
pub struct PrintJobORM {
    /// Stable job identifier
    #[ormlite(primary_key)]
    pub uid: Uuid,

    /// Owning user
    #[ormlite(column = "user")]
    pub user: Join<User>,

    /// Target queue (capability-based)
    pub queue: Uuid,

    /// Original filename or description
    pub name: String,
    /// the file being printed (PDF, SVG, STL, OBJ, GCODE, etc.)
    #[ormlite(column = "asset")]
    pub asset: Join<AssetORM>,

    /// Estimated cost (computed at submission)
    pub estimated_cost: Option<f64>,

    /// Job creation timestamp
    pub created_at: i64,

    /// Terminal lifecycle state
    pub state: String,

    /// When job entered terminal state
    pub completed_at: Option<i64>,

    pub copies: Option<i32>,
    pub priority: Option<String>,

    // =========================
    // Advanced Settings
    // =========================
    pub material: Option<String>,
    pub nozzle_temp: Option<i32>,
    pub bed_temp: Option<i32>,
    pub layer_height: Option<f32>,
    pub infill: Option<i32>,
    pub supports: Option<String>,
    pub notes: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
pub struct PrintQueueORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub last_active: i64,
    /// has the queue been advertised to the network
    pub advertised: bool,
}

impl PrintQueueORM {
    pub fn new(name: String) -> Self {
        Self {
            uid: Uuid::new_v4(),
            name,
            description: None,
            created_at: unix_epoch_seconds(),
            last_active: unix_epoch_seconds(),
            advertised: false,
        }
    }

    pub fn with_description(name: String, description: String) -> Self {
        let mut queue = Self::new(name);
        queue.description = Some(description);
        queue
    }
}
