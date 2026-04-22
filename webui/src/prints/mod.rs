pub mod routes;
pub mod forms;

use uuid::Uuid;
use ormlite::Model;
use serde_derive::{Deserialize, Serialize};

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
    /// the file being printed (PDF, SVG, STL, OBJ, GCODE, etc.)
    pub asset: Uuid,

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
