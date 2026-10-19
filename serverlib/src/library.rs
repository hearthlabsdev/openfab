use crate::errors::{DownloadErr, OpenFabErr, UploadErr};
use crate::utils::unix_epoch_seconds;
use crate::utils::{FileID, ResultStream};

use tokio::io::AsyncReadExt;

use blake3;
use hex;
use minio_rsc::http::Method;
use ormlite::model::JoinMeta;
use ormlite::postgres::PgPool;
use ormlite::{Model, model::Join};
use rocket::fs::TempFile;
use secret_ref::*;
use sha2::{Digest, Sha256};

pub const PAGE_SIZE: usize = 20;

use minio_rsc::{Minio, client::PresignedArgs, provider::StaticProvider};
use rocket_okapi::okapi::schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileInfo {
    pub name: String,
    pub size: i64,
    pub hash_alg: String,
    pub hash: String,
    pub mime: String,
    pub modified: i64,
}

#[derive(Debug, Model, Clone, Serialize, Deserialize, JsonSchema)]
#[ormlite(table = "libraries")]
pub struct LibraryORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,
    pub owner: Uuid,

    pub created_at: i64,
    pub updated_at: i64,

    /// Whether this asset is publicly accessible (e.g. for embedding or sharing)
    pub public: bool,
}

impl LibraryORM {
    pub fn new(name: &str, owner: Uuid, public: bool) -> Self {
        Self {
            uid: Uuid::new_v4(),
            name: name.into(),
            owner,
            created_at: unix_epoch_seconds(),
            updated_at: unix_epoch_seconds(),
            public,
        }
    }

    pub async fn list(pool: &PgPool) -> Result<Vec<LibraryORM>, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        Ok(LibraryORM::select().fetch_all(&mut *conn).await?)
    }

    pub async fn list_assets(
        pool: &PgPool,
        uid: Uuid,
        page: Option<i64>,
    ) -> Result<Vec<AssetORM>, OpenFabErr> {
        let page: usize = (page.unwrap_or(1) - 1).abs().try_into().unwrap_or(0);
        let mut conn = pool.acquire().await?;
        Ok(AssetORM::select()
            .where_("library = ?")
            .bind(uid)
            .order_desc("created_at")
            .limit(PAGE_SIZE)
            .offset(page * PAGE_SIZE)
            .fetch_all(&mut *conn)
            .await?)
    }

    pub async fn get_asset(pool: &PgPool, uid: Uuid) -> Result<AssetORM, OpenFabErr> {
        let mut conn = pool.acquire().await?;
        Ok(AssetORM::select()
            .where_("uid = ?")
            .bind(uid)
            .fetch_one(&mut *conn)
            .await?)
    }
}

/// resource represents the actual model assets are anything associated with the model
#[derive(Debug, Serialize, Deserialize, Model)]
#[ormlite(table = "resources")]
pub struct ResourceORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,
    /// ormlite has yet to impl one to many joins, so for now this has to be done manually
    pub images: Vec<Uuid>,

    #[ormlite(column = "asset")]
    pub asset: Join<AssetORM>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uid: Uuid,
    pub name: String,
    pub images: Vec<AssetORM>,
    pub asset: AssetORM,
}

#[derive(Debug, Model, Clone, Serialize, Deserialize, JsonSchema)]
#[ormlite(table = "assets")]
pub struct AssetORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,
    pub library: Uuid,
    pub mime: String,
    pub key: String,
    /// Owning user (for permissions and sharing)
    pub owner: Uuid,

    pub hash_alg: String,
    pub hash: String,

    /// Size of the file in bytes.
    pub size: i64,

    pub created_at: i64,
    /// when the file was last modified
    pub updated_at: i64,
    /// Last time this asset was used in a print job or queue.
    pub last_used_at: Option<i64>,

    /// Whether this asset is publicly accessible (e.g. for embedding or sharing)
    pub public: bool,
    /// Optional license or usage terms (e.g. "CC-BY-SA 4.0")
    pub license: Option<String>,
}

impl AssetORM {
    pub async fn download_bytes(&self, store: &ObjectStore) -> Result<Vec<u8>, DownloadErr> {
        // should this update last used?
        Ok(store.download_bytes(&self.key).await?)
    }
}

impl FileID for AssetORM {
    fn hash_alg(&self) -> &str {
        &self.hash_alg
    }
    fn hash(&self) -> &str {
        &self.hash
    }
    fn size(&self) -> i64 {
        self.size
    }
    fn modified(&self) -> i64 {
        self.updated_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectStoreConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: SecretRef,
    pub bucket: String,
    pub secure: bool,
    /// primary algorithm name, e.g. "sha256" or "blake3"
    pub primary_hash_alg: Option<String>,
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
            self.primary_hash_alg
                .as_ref()
                .map_or("sha256", |v| v.as_str()),
        )
    }
}

/*
pub struct StorageBackend {
    S3(ObjectStore),
    FTPS(TokioRustlsFtpStream),

}*/

pub struct ObjectStore {
    client: Minio,
    bucket: String,
    /// primary algorithm name, e.g. "sha256" or "blake3"
    primary_hash_alg: String,
}

impl ObjectStore {
    /// Build a new client pointed at your MinIO server.
    pub fn new(
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
        bucket: &str,
        secure: bool,
        primary_hash_alg: &str,
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
            primary_hash_alg: primary_hash_alg.to_string(),
        })
    }

    /// Upload raw bytes (server‑side upload).
    pub async fn upload_bytes(
        &self,
        key: &str,
        data: Vec<u8>,
    ) -> Result<(String, String), UploadErr> {
        let blake3_hex = blake3::hash(&data).to_hex().to_string();
        // Compute SHA-256 hex
        let mut sha_hasher = Sha256::new();
        sha_hasher.update(&data);
        let sha256_hex = hex::encode(sha_hasher.finalize());

        // Choose the algorithm to report according to configuration (fallback to sha256)
        let chosen_alg = if self.primary_hash_alg.is_empty() {
            "sha256"
        } else {
            self.primary_hash_alg.as_str()
        };

        let chosen_hash = match chosen_alg {
            "blake3" => blake3_hex.clone(),
            "sha256" => sha256_hex.clone(),
            other => {
                // unknown algorithm -> fallback to sha256 but still return the configured name
                eprintln!(
                    "warning: unknown hash algorithm `{}`, falling back to sha256",
                    other
                );
                sha256_hex.clone()
            }
        };
        self.client
            .put_object(&self.bucket, key, data.into())
            .await
            .expect("failed to put object");

        Ok((chosen_alg.into(), chosen_hash))
    }

    pub async fn upload_temp_file(
        &self,
        key: &str,
        tempfile: &TempFile<'_>,
    ) -> Result<FileInfo, UploadErr> {
        let path = tempfile.path().ok_or(UploadErr::InvalidPath)?;
        let mime = tempfile
            .content_type()
            .map(|v| v.0.to_string())
            .ok_or(UploadErr::IndeterminateFileType)?;
        // Read file bytes (upload_bytes expects Vec<u8> anyway)
        let data = tokio::fs::read(&path).await?;

        let data = tokio::fs::read(tempfile.path().unwrap()).await?;

        let len = data.len();

        let (hash_alg, hash) = self.upload_bytes(key, data).await?;
        let file_info = FileInfo {
            name: key.to_string(),
            size: len as i64,
            hash,
            hash_alg,
            mime,
            modified: unix_epoch_seconds(),
        };

        Ok(file_info)
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
        let args = PresignedArgs::new(&self.bucket, key).expires(expires_secs);

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
        let args = PresignedArgs::new(&self.bucket, key).expires(expires_secs);

        let url = self.client.presigned_get_object(args).await?;
        Ok(url)
    }

    pub async fn download_bytes(&self, key: &str) -> Result<Vec<u8>, DownloadErr> {
        let mut buf = Vec::new();
        self.get_streamed(key).await?.read_to_end(&mut buf).await?;
        Ok(buf)
    }

    pub async fn get_streamed(&self, key: impl Into<String>) -> Result<ResultStream, DownloadErr> {
        let res = self
            .client
            .executor(Method::GET)
            .bucket_name(&self.bucket)
            .object_name(key.into())
            .send_ok()
            .await?;

        let ct_type = res
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        // Map reqwest::Error -> std::io::Error using TryStreamExt
        let io_stream = res.bytes_stream();

        // Boxing it makes the complex return signature clean and satisfies Rocket's 'static bounds
        Ok(ResultStream::new(io_stream, ct_type))
    }
}
