use crate::status::Extensions;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub size: i64,
    pub modified: i64,
    pub path: PathBuf,
    #[serde(flatten)]
    extras: HashMap<String, Value>,
}

impl FileEntry {
    /// Creates a new [`FileEntry`] by combining core file info with a
    /// strongly-typed vendor extension struct.
    ///
    /// # Errors
    /// Returns a [`serde_json::Error`] if the provided `extras` extension struct fails
    /// to serialize into a valid JSON object map.
    pub fn with_extras<E: Extensions, P: AsRef<Path>>(
        name: impl Into<String>,
        path: P,
        modified: i64,
        size: i64,
        extra: E,
    ) -> Result<Self, serde_json::Error> {
        let val = serde_json::to_value(extra)?;
        let mut extras: HashMap<String, Value> = serde_json::from_value(val)?;
        extras.retain(|_, v| !v.is_null());

        Ok(Self {
            name: name.into(),
            path: path.as_ref().to_path_buf(),
            size,
            modified,
            extras,
        })
    }

    pub fn new<P: AsRef<Path>>(name: impl Into<String>, path: P, size: i64, modified: i64) -> Self {
        Self {
            name: name.into(),
            path: path.as_ref().to_path_buf(),
            modified,
            size,
            extras: HashMap::new(),
        }
    }
}
