pub mod ui;
use crate::errors::OpenFabErr;
use ormlite::Model;
use rocket::fs::TempFile;
use secret_ref::*;
use serde_derive::{Deserialize, Serialize};
pub use ui::get_routes;
use uuid::Uuid;

use minio_rsc::{Minio, client::PresignedArgs, provider::StaticProvider};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
#[ormlite(table = "libraries")]
pub struct LibraryORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,
    pub owner: Uuid,

    pub created_at: i64,
    pub updated_at: i64,
    /// Last time this asset was used in a print job or queue.
    pub last_used_at: Option<i64>,

    /// Whether this asset is publicly accessible (e.g. for embedding or sharing)
    pub public: bool,
    /// Optional license or usage terms (e.g. "CC-BY-SA 4.0")
    pub license: Option<String>,
}

#[derive(Debug, Model, Clone, Serialize, Deserialize)]
#[ormlite(table = "assets")]
pub struct AssetORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,
    pub mime: String,
    /// Owning user (for permissions and sharing)
    pub owner: Uuid,

    /// Size of the file in bytes.
    pub size: i64,

    pub created_at: i64,
    pub updated_at: i64,
    /// Last time this asset was used in a print job or queue.
    pub last_used_at: Option<i64>,

    /// Whether this asset is publicly accessible (e.g. for embedding or sharing)
    pub public: bool,
    /// Optional license or usage terms (e.g. "CC-BY-SA 4.0")
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectStoreConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: SecretRef,
    pub bucket: String,
    pub secure: bool,
}

impl ObjectStoreConfig {
    pub fn is_setup(&self) -> bool {
        self.endpoint != "" && self.access_key != "" && self.bucket != ""
    }

    pub async fn object_store(&self) -> Result<ObjectStore, OpenFabErr> {
        ObjectStore::new(
            &self.endpoint,
            &self.access_key,
            &self
                .secret_key
                .fetch(SecretPolicy::default())
                .await?
                .expose(),
            &self.bucket,
            self.secure,
        )
    }
}

pub struct ObjectStore {
    client: Minio,
    bucket: String,
}

impl ObjectStore {
    /// Build a new client pointed at your MinIO server.
    pub fn new(
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
        bucket: &str,
        secure: bool,
    ) -> Result<Self, OpenFabErr> {
        let provider = StaticProvider::new(access_key, secret_key, None);
        let client = Minio::builder()
            .endpoint(endpoint)
            .provider(provider)
            .secure(secure)
            .build()?;

        Ok(ObjectStore {
            client,
            bucket: bucket.to_string(),
        })
    }

    /// Upload raw bytes (server‑side upload).
    pub async fn upload_bytes(
        &self,
        key: &str,
        data: Vec<u8>,
    ) -> Result<(), minio_rsc::error::Error> {
        self.client
            .put_object(&self.bucket, key, data.into())
            .await
            .expect("failed to put object");
        Ok(())
    }

    pub async fn upload_temp_file(
        &self,
        key: &str,
        tempfile: &TempFile<'_>,
    ) -> Result<(), minio_rsc::error::Error> {
        let data = tokio::fs::read(tempfile.path().unwrap()).await?;
        self.upload_bytes(key, data).await?;
        Ok(())
    }

    /// Generate a presigned PUT URL.
    ///
    /// Frontends can upload using:
    ///   `PUT <generated_url>` with the file body.
    pub async fn presigned_put_url(
        &self,
        key: &str,
        expires_secs: usize,
    ) -> Result<String, OpenFabErr> {
        // build presign args
        let mut args = PresignedArgs::new(&self.bucket, key).expires(expires_secs);

        // generate signed URL
        let url = self.client.presigned_put_object(args).await?;
        Ok(url)
    }

    /// Generate a presigned GET URL.
    pub async fn presigned_get_url(
        &self,
        key: &str,
        expires_secs: usize,
    ) -> Result<String, OpenFabErr> {
        let mut args = PresignedArgs::new(&self.bucket, key).expires(expires_secs);

        let url = self.client.presigned_get_object(args).await?;
        Ok(url)
    }
}
