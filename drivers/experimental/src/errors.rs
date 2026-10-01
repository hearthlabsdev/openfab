use thiserror::Error;
use wasmi::errors::LinkerError;

#[derive(Debug, Error)]
pub enum DriverErr {
    #[error("wasmi error: {0}")]
    Wasmi(#[from] wasmi::Error),
    #[error("module not found")]
    ModuleNotFound,
    #[error("linking failure: {0}")]
    LinkErr(#[from] LinkerError),
}