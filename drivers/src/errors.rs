#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("configuration error: {0}")]
    Configuration(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("device error: {0}")]
    Device(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),

    #[error("couldn't find driver")]
    DriverNotFound,
}