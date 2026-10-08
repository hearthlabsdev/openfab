pub mod forms;
pub mod materials;
pub mod ui;
use crate::devices::DeviceModelORM;
use crate::errors::OpenFabErr;
use crate::library::{AssetORM, ObjectStore};
use ormlite::Model;
use ormlite::postgres::PgPool;
use serde::{Deserialize, Serialize};
pub use ui::get_routes;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "settings")]
pub struct SettingsORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub user: Option<Uuid>,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceModelInfo {
    pub uid: Uuid,
    pub name: String,
    pub variants: Vec<String>,
    pub technology: Option<String>,
    pub family: Option<String>,
    pub url: Option<String>,
    pub default_materials: Vec<String>,
}

impl DeviceModelInfo {
    pub async fn from_device_model_orm(
        pool: &PgPool,
        store: &ObjectStore,
        orm: DeviceModelORM,
    ) -> Result<Self, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        let url = match orm.thumbnail {
            Some(asset_id) => {
                let asset = AssetORM::select()
                    .where_("uid = ?")
                    .bind(asset_id)
                    .fetch_one(&mut *conn)
                    .await?;
                Some(store.presigned_get_url(&asset.key, 60 * 60).await?)
            }
            None => None,
        };

        Ok(Self {
            uid: orm.uid,
            name: orm.name,
            variants: orm.variants,
            technology: orm.technology,
            family: orm.family,
            url,
            default_materials: orm.default_materials,
        })
    }
}
