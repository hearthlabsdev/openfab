use ormlite::Model;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "filaments")]
pub struct FilamentORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,
    pub vendor: Option<String>,
    pub cost: Option<f64>,
    pub colour: String,
    pub density: Option<String>,
    pub spool_weight: Option<String>,
    pub filament_type: Option<String>,
    pub notes: Option<String>,
}
