use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use openfab::discovery::IppDiscoveryError;
use crate::discovery::AdvertiseCmd;

#[derive(Debug, Error)]
pub enum OpenMPDErr {
    #[error("failed to send advertisement command: {0}")]
    AdvertiseErr(#[from] SendError<AdvertiseCmd>),
    #[error("failed to discover printers: {0}")]
    IppDiscovery(#[from] IppDiscoveryError),
    #[error("database error: {0}")]
    OrmLite(#[from] ormlite::Error),
    #[error("sql error: {0}")]
    SQLXErr(#[from] ormlite::SqlxError),
}