use crate::prints::AssetORM;
use crate::prints::PrintJobORM;
use crate::prints::User;
use crate::utils::unix_epoch_seconds;
use ormlite::model::Join;
use rocket::form::FromForm;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct PrintForm {
    pub asset: Option<Uuid>,
    //pub upload: Option<TempFile<'r>>,
    pub device: Option<Uuid>,
    pub queue: Option<Uuid>,
    pub name: String,
    pub copies: i32,
    pub priority: String,

    // ================================================
    //              Advanced Configuration
    // ================================================
    // most of these fields don't do anything as of yet.
    /// should eventually be switched to using a UUID once material DB is built out.
    pub material: Option<String>,
    pub nozzle_temp: Option<i32>,
    pub bed_temp: Option<i32>,
    pub layer_height: Option<f32>,
    pub infill: Option<i32>,
    pub supports: Option<String>,
    pub support_type: Option<String>,
    pub notes: Option<String>,
}

impl PrintForm {
    pub fn into_print_job(self, user: User, asset: AssetORM, queue: Uuid) -> PrintJobORM {
        PrintJobORM {
            uid: Uuid::new_v4(),
            user: Join::new(user),
            queue,
            name: self.name,
            asset: Join::new(asset),
            estimated_cost: None,
            created_at: unix_epoch_seconds(),
            state: "pending".into(),
            completed_at: None,
            copies: Some(self.copies),
            priority: Some(self.priority),

            // =========================
            // Advanced Settings
            // =========================
            material: self.material,
            nozzle_temp: self.nozzle_temp,
            bed_temp: self.bed_temp,
            layer_height: self.layer_height,
            infill: self.infill,
            supports: self.supports,
            notes: self.notes,
        }
    }
}
