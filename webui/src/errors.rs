use crate::discovery::AdvertiseCmd;
use openfab::discovery::IppDiscoveryError;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, Error)]
pub enum OpenFabErr {
    #[error("failed to send advertisement command: {0}")]
    AdvertiseErr(#[from] SendError<AdvertiseCmd>),
    #[error("failed to discover printers: {0}")]
    IppDiscovery(#[from] IppDiscoveryError),
    #[error("database error: {0}")]
    OrmLite(#[from] ormlite::Error),
    #[error("sql error: {0}")]
    SQLXErr(#[from] ormlite::SqlxError),
    #[error("Minio error: {0}")]
    MinioErr(#[from] minio_rsc::error::Error),
    #[error("Minio value error: {0}")]
    ValueErr(#[from] minio_rsc::error::ValueError),
    #[error("secret fetch error: {0}")]
    SecretFetchErr(#[from] secret_ref::SecretError),
    #[error("authentication failed: {0}")]
    AuthErr(#[from] AuthErr),
    #[error("failed to parse string to uuid: {0}")]
    UuidParseErr(#[from] uuid::Error),
}

#[derive(Debug, Error)]
pub enum AuthErr {
    #[error("invalid password entry")]
    InvalidPassword,
    #[error("user not found in database")]
    UserNotFound,
}
