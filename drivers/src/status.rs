//! # Printer Status and Telemetry Module
//!
//! This module provides core, vendor-agnostic data structures for representing 3D printer
//! telemetry, job progress, and upload status. 
//!
//! To support brand-specific features (such as Prusa-specific storage/transfers or Bambu-specific 
//! AMS states) without introducing generic parameters into application-level event loops, 
//! [`PrinterStatus`] encapsulates unknown or vendor-specific fields within a private, flattened 
//! hash map while offering type-safe constructor validation via the [`Extensions`] trait.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

/// A marker trait representing types that can be safely serialized and deserialized
/// as vendor-specific printer extension payloads.
///
/// This trait automatically blankets any type that implements [`Debug`], [`Clone`],
/// [`Serialize`], and [`DeserializeOwned`].
pub trait Extensions: Debug + Clone + Serialize + DeserializeOwned {}

impl<T: Debug + Clone + Serialize + DeserializeOwned> Extensions for T {}

/// Represents the current operational state of a 3D printer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum State {
    /// The printer is idle and waiting for instructions.
    Idle,
    /// The printer is currently busy executing internal routines or preparations.
    Busy,
    /// The printer is actively printing a job.
    Printing,
    /// The printing process has been temporarily paused.
    Paused,
    /// The print job has successfully completed.
    Finished,
    /// The print job was stopped or aborted prematurely.
    Stopped,
    /// The printer has encountered a fatal error.
    Error,
    /// The printer requires user attention or intervention (e.g., filament change or sensor warning).
    Attention,
    /// The printer is fully initialized, online, and ready for work.
    Ready,
}

/// Core, vendor-agnostic status metrics reported by a 3D printer.
///
/// Contains universal telemetry such as temperatures, alongside an encapsulated,
/// flattened `extra` map for vendor-specific properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterStatus {
    /// Current measured temperature of the heated bed in degrees Celsius.
    pub bed_temp: f64,
    /// Target temperature of the heated bed, if active.
    pub target_bed_temp: Option<f64>,
    /// Current measured temperature of the print head nozzle in degrees Celsius.
    pub nozzle_temp: f64,
    /// Target temperature of the print head nozzle, if active.
    pub target_nozzle_temp: Option<f64>,
    /// Current measured temperature of the print chamber (if equipped).
    pub chamber_temp: Option<f64>,
    /// Private catch-all for vendor-specific JSON fields, flattened during serialization.
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl PrinterStatus {
    /// Creates a new [`PrinterStatus`] by combining core temperature telemetry with a 
    /// strongly-typed vendor extension struct.
    ///
    /// # Errors
    /// Returns a [`serde_json::Error`] if the provided `extra` extension struct fails 
    /// to serialize into a valid JSON object map.
    pub fn with_extras<E: Extensions>(
        bed_temp: f64,
        target_bed_temp: Option<f64>,
        nozzle_temp: f64,
        target_nozzle_temp: Option<f64>,
        chamber_temp: Option<f64>,
        extra: E,
    ) -> Result<Self, serde_json::Error> {
        let val = serde_json::to_value(extra)?;
        let mut extras: HashMap<String, Value> = serde_json::from_value(val)?;
        extras.retain(|_, v| !v.is_null());

        Ok(Self {
            bed_temp,
            target_bed_temp,
            nozzle_temp,
            target_nozzle_temp,
            chamber_temp,
            extra: extras,
        })
    }
}

/// Information tracking the progress and timing of an active print job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {

    /// Estimated time remaining until job completion, measured in seconds.
    pub time_remaining_secs: i32,
    /// Current progress of the print job represented as a percentage (0 to 100).
    pub progress: i32,

    pub gcode_state: Option<String>,
    /// Private catch-all for vendor-specific JSON fields, flattened during serialization.
    extras: HashMap<String, Value>,
}

impl JobStatus {
    pub fn with_extras<E: Extensions>(time_remaining_secs: i32, progress: i32, gcode_state: Option<String>, extra: E) -> Result<Self, serde_json::Error> {
        let val = serde_json::to_value(extra)?;
        let mut extras: HashMap<String, Value> = serde_json::from_value(val)?;
        extras.retain(|_, v| !v.is_null());
        Ok(
            Self {
                time_remaining_secs,
                progress,
                gcode_state,
                extras
            }
        )
    }
}

/// Status telemetry regarding a file upload operation sent to the printer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadStatus {
    /// Current progress percentage of the file upload (0 to 100).
    pub progress: i32,
    /// Total data transferred so far, measured in bytes (if available).
    pub data_tx: Option<i32>,
    /// Optional status or error message related to the upload.
    pub message: Option<String>,
}

impl std::cmp::PartialEq for UploadStatus {
    /// Compares two [`UploadStatus`] instances strictly based on their upload `progress`.
    fn eq(&self, other: &UploadStatus) -> bool {
        self.progress == other.progress
    }
}