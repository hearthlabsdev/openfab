use uuid::Uuid;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrintPriority {
    Low,
    Normal,
    High,
}

impl Default for PrintPriority {
    fn default() -> Self {
        PrintPriority::Normal
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupportType {
    None,
    Buildplate,
    Everywhere,
    Tree,
    Linear,
}


use rocket::form::{Form, FromForm};
use rocket::fs::TempFile;

#[derive(Debug, FromForm)]
pub struct PrintJobUpload<'r> {
    pub device_id: String,
    pub job_name: Option<String>,
    pub copies: Option<i32>,
    pub priority: Option<String>,

    pub material: Option<String>,
    pub nozzle_temp: Option<i32>,
    pub bed_temp: Option<i32>,
    pub layer_height: Option<f32>,
    pub infill: Option<i32>,
    pub supports: Option<String>,
    pub notes: Option<String>,

    pub file: TempFile<'r>,
}