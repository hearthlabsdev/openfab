use thiserror::Error;

#[derive(Debug, Error)]
pub enum SDKError {
    #[error("failed to send request: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("missing argument {0} in command processor")]
    MissingArgument(String),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("other error occured: {0}")]
    Other(String),
}

impl From<String> for SDKError {
    fn from(val: String) -> Self {
        Self::Other(val)
    }
}