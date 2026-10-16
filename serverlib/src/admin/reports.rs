use ormlite::Model;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "bug_reports")]
pub struct BugReportORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub error: String,
    pub route: Option<String>,
    pub description: Option<String>,
    pub submitted: i64,
}
