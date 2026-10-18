use crate::accounts::User;
use crate::devices::DeviceORM;
use crate::errors::OpenFabErr;
use crate::library::AssetORM;
use crate::utils::unix_epoch_seconds;
use openfab_drivers::status::JobStatus;
use ormlite::Model;
use ormlite::model::Join;
use ormlite::model::JoinMeta;
use ormlite::postgres::PgPool;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use rocket_okapi::okapi::schemars::JsonSchema;

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

    pub priority: Option<String>,

    /// number of copies to print
    /// this should be decremented for each successful print instance.
    pub copies: Option<i32>,

    /// Job creation timestamp
    pub created_at: i64,

    /// Terminal lifecycle state
    pub state: String,
    /// keeps track of instance number
    pub count: i32,

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

impl PrintJobORM {
    pub async fn admin_get_print(pool: &PgPool, uid: Uuid) -> Result<PrintJobORM, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        Ok(PrintJobORM::select()
            .join(PrintJobORM::user())
            .join(PrintJobORM::asset())
            .where_("uid = ?")
            .bind(uid)
            .fetch_one(&mut *conn)
            .await?)
    }
    pub async fn get_print(pool: &PgPool, uid: Uuid) -> Result<PrintJob, OpenFabErr> {
        let orm = Self::admin_get_print(pool, uid).await?;
        Ok(orm.into())
    }
    pub async fn start(
        &mut self,
        device: Uuid,
        conn: &mut ormlite::postgres::PgConnection,
    ) -> Result<PrintJobInstance, OpenFabErr> {
        let instance = PrintJobInstanceORM::create_instance(self, device, conn).await?;
        self.count += 1;
        unimplemented!();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrintJob {
    /// Stable job identifier
    pub uid: Uuid,

    /// Owning user
    pub user: Uuid,

    /// Target queue (capability-based)
    pub queue: Uuid,

    /// Original filename or description
    pub name: String,
    /// the file being printed (PDF, SVG, STL, OBJ, GCODE, etc.)
    pub asset: Uuid,

    /// Estimated cost (computed at submission)
    pub estimated_cost: Option<f64>,

    /// Job creation timestamp
    pub created_at: i64,

    /// Terminal lifecycle state
    pub state: String,

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrintJobStatus {
    /// Estimated time remaining until job completion, measured in seconds.
    pub time_remaining_secs: i64,
    /// Current progress of the print job represented as a percentage (0 to 100).
    pub progress: i64,

    pub state: String,
    /// Private catch-all for vendor-specific JSON fields, flattened during serialization.
    #[serde(flatten)]
    extras: HashMap<String, Value>,
}

impl From<JobStatus> for PrintJobStatus {
    fn from(status: JobStatus) -> Self {
        Self {
            time_remaining_secs: status.time_remaining_secs,
            progress: status.progress,
            state: status.state.to_string(),
            extras: status.extras,
        }
    }
}

impl From<PrintJobORM> for PrintJob {
    fn from(orm: PrintJobORM) -> Self {
        Self {
            uid: orm.uid,
            user: orm.user.id,
            queue: orm.queue,
            name: orm.name,
            asset: orm.asset.id,
            estimated_cost: orm.estimated_cost,
            created_at: orm.created_at,
            state: orm.state,
            copies: orm.copies,
            priority: orm.priority,
            material: orm.material,
            nozzle_temp: orm.nozzle_temp,
            bed_temp: orm.bed_temp,
            layer_height: orm.layer_height,
            infill: orm.infill,
            supports: orm.supports,
            notes: orm.notes,
        }
    }
}
/*
impl From<PrintJob> for PrintJobORM {
    fn from(job: PrintJob) -> Self {
        Self {
            uid: job.uid,
            user: Join::new(job.user),
            queue: job.queue,
            name: job.name,
            asset: Join::new(job.asset),
            estimated_cost: job.estimated_cost,
            created_at: job.created_at,
            state: job.state,
            copies: job.copies,
            priority: job.priority,
            material: job.material,
            nozzle_temp: job.nozzle_temp,
            bed_temp: job.bed_temp,
            layer_height: job.layer_height,
            infill: job.infill,
            supports: job.supports,
            notes: job.notes,
        }
    }
}*/

#[derive(Debug, Clone, Serialize, Deserialize, Model, JsonSchema)]
#[ormlite(table = "print_job_executions")]
pub struct PrintJobInstanceORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub job: Uuid,
    pub printer: Uuid,
    pub instance: i32,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub error: Option<String>,
}

impl PrintJobInstanceORM {
    fn create(job: &PrintJobORM, device: Uuid) -> Self {
        Self {
            uid: Uuid::new_v4(),
            job: job.uid,
            printer: device,
            instance: job.count + 1,
            started_at: Some(unix_epoch_seconds()),
            finished_at: None,
            error: None,
        }
    }
    pub(crate) async fn create_instance(
        job: &mut PrintJobORM,
        device: Uuid,
        conn: &mut ormlite::postgres::PgConnection,
    ) -> Result<PrintJobInstanceORM, OpenFabErr> {
        let instance = Self::create(job, device);
        Ok(instance.insert(&mut *conn).await?)
    }
    pub async fn list_events(
        pool: &PgPool,
        job_instance: Uuid,
    ) -> Result<Vec<PrintJobEvent>, OpenFabErr> {
        let mut conn = pool.acquire().await?;

        Ok(PrintJobEventORM::select()
            .where_("job_instance = ?")
            .bind(job_instance)
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .map(|v| v.try_into())
            .collect::<Result<Vec<PrintJobEvent>, OpenFabErr>>()?)
    }
    /// this would be a more expensive query bc of ormlites current limitations so instead this provides a simple index of job instances.
    pub async fn list_job_instances(
        pool: &PgPool,
        job: Uuid,
    ) -> Result<Vec<PrintJobInstanceORM>, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        Ok(PrintJobInstanceORM::select()
            .where_("job = ?")
            .bind(job)
            .fetch_all(&mut *conn)
            .await?)
    }

    /// because this is an expensive query that has to be done manually for now, the full information is only available for single objects.
    /// querying user information as an extraneous requirements will also for now not be supported without explicitly requesting the user information seperately.
    pub async fn get_job_instance(
        pool: &PgPool,
        uid: Uuid,
    ) -> Result<PrintJobInstance, OpenFabErr> {
        let print_job = PrintJobORM::get_print(pool, uid).await?;

        let mut conn = pool.acquire().await?;
        let orm = PrintJobInstanceORM::select()
            .where_("job = ?")
            .bind(uid)
            .fetch_one(&mut *conn)
            .await?;

        let device = DeviceORM::select()
            .where_("uid = ?")
            .bind(orm.printer)
            .fetch_one(&mut *conn)
            .await?;

        let events = PrintJobInstanceORM::list_events(pool, orm.uid).await?;

        Ok(PrintJobInstance {
            uid: orm.uid,
            job: print_job,
            printer: device,
            instance: orm.instance,
            events,
            started_at: orm.started_at,
            finished_at: orm.finished_at,
            error: orm.error,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrintJobInstance {
    pub uid: Uuid,
    pub job: PrintJob,
    pub printer: DeviceORM,
    pub instance: i32,
    pub events: Vec<PrintJobEvent>,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
#[ormlite(table = "print_job_events")]
pub struct PrintJobEventORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,

    /// Parent job
    pub job_instance: Uuid,

    /// Execution state
    pub status: String,

    /// When the event was recorded
    pub timestamp: i64,
    /// Failure reason (if any)
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrintJobEvent {
    pub uid: Uuid,
    pub job_instance: Uuid,
    pub status: PrintJobStatus,
    pub error: Option<String>,
    pub timestamp: i64,
}

impl TryFrom<PrintJobEventORM> for PrintJobEvent {
    type Error = OpenFabErr;

    fn try_from(orm: PrintJobEventORM) -> Result<Self, Self::Error> {
        Ok(Self {
            uid: orm.uid,
            job_instance: orm.job_instance,
            status: serde_json::from_str(&orm.status)?,
            timestamp: orm.timestamp,
            error: orm.error,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Model, JsonSchema)]
#[ormlite(table = "print_queues")]
pub struct PrintQueueORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub last_active: i64,
    pub mime: Option<Vec<String>>,
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
            mime: None,
        }
    }

    pub fn with_description(name: String, description: String) -> Self {
        let mut queue = Self::new(name);
        queue.description = Some(description);
        queue
    }
    pub async fn list(pool: &PgPool) -> Result<Vec<PrintQueueORM>, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        Ok(PrintQueueORM::select().fetch_all(&mut *conn).await?)
    }

    pub async fn get_queue(pool: &PgPool, uid: Uuid) -> Result<PrintQueueORM, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        Ok(PrintQueueORM::select()
            .where_("uid = ?")
            .bind(uid)
            .fetch_one(&mut *conn)
            .await?)
    }

    pub async fn list_prints(pool: &PgPool, queue: Uuid) -> Result<Vec<PrintJob>, OpenFabErr> {
        Ok(Self::admin_list_prints(pool, queue)
            .await?
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<PrintJob>>())
    }
    pub async fn admin_list_prints(
        pool: &PgPool,
        queue: Uuid,
    ) -> Result<Vec<PrintJobORM>, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        Ok(PrintJobORM::select()
            .join(PrintJobORM::user())
            .join(PrintJobORM::asset())
            .where_("queue = ?")
            .bind(queue)
            .fetch_all(&mut *conn)
            .await?)
    }
}
