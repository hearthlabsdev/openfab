#![deny(missing_docs)]
//! # Printer Status and Telemetry Module
//!
//! This module provides core, vendor-agnostic data structures for representing 3D printer
//! telemetry, job progress, and upload status.
//!
//! To support brand-specific features (such as Prusa-specific storage/transfers or Bambu-specific
//! AMS states) without introducing generic parameters into application-level event loops,
//! [`PrinterStatus`] encapsulates unknown or vendor-specific fields within a private, flattened
//! hash map while offering type-safe constructor validation via the [`Extensions`] trait.

use crate::EventType;
use approx::relative_eq;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
use std::convert::Infallible;

/// A marker trait representing types that can be safely serialized and deserialized
/// as vendor-specific printer extension payloads.
///
/// This trait automatically blankets any type that implements [`Debug`], [`Clone`],
/// [`Serialize`], and [`DeserializeOwned`].
pub trait Extensions: Debug + Clone + Serialize + DeserializeOwned {}

impl<T: Debug + Clone + Serialize + DeserializeOwned> Extensions for T {}

/// Represents the current operational state of a 3D printer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    /// the print is offline.
    Offline,
    /// the printer is slicing a file
    Slicing,
    /// The printer has encountered a fatal error.
    Error,
    /// The printer requires user attention or intervention (e.g., filament change or sensor warning).
    Attention,
    /// The printer is fully initialized, online, and ready for work.
    Ready,
    /// catch all
    Unknown(String),
}

impl State {
    /// matches against State::Ready || State::Idle to determine readyness status for a new print job.
    pub fn is_ready(&self) -> bool {
        *self == State::Ready || *self == State::Idle
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Idle => write!(f, "Idle"),
            State::Busy => write!(f, "Busy"),
            State::Printing => write!(f, "Printing"),
            State::Paused => write!(f, "Paused"),
            State::Finished => write!(f, "Finished"),
            State::Stopped => write!(f, "Stopped"),
            State::Error => write!(f, "Error"),
            State::Attention => write!(f, "Attention"),
            State::Ready => write!(f, "Ready"),
            State::Slicing => write!(f, "Slicing"),
            State::Offline => write!(f, "Offline"),
            State::Unknown(s) => write!(f, "Unknown({})", s),
        }
    }
}

impl FromStr for State {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_ascii_lowercase();

        match lower.as_str() {
            "idle" => Ok(State::Idle),
            "busy" => Ok(State::Busy),
            "printing" => Ok(State::Printing),
            "paused" => Ok(State::Paused),
            "finished" => Ok(State::Finished),
            "stopped" => Ok(State::Stopped),
            "error" => Ok(State::Error),
            "attention" => Ok(State::Attention),
            "ready" => Ok(State::Ready),
            "slicing" => Ok(State::Slicing),
            "offline" => Ok(State::Offline),
            _ => {
                // If it starts with "unknown(" (case-insensitive check), peel out the inner value
                if let Some(inner) = s.strip_prefix("Unknown(")
                    .or_else(|| s.strip_prefix("unknown("))
                    .and_then(|rem| rem.strip_suffix(')')) 
                {
                    Ok(State::Unknown(inner.to_string()))
                } else {
                    Ok(State::Unknown(s.to_string()))
                }
            }
        }
    }
}

/// Core, vendor-agnostic status metrics reported by a 3D printer.
///
/// Contains universal telemetry such as temperatures, alongside an encapsulated,
/// flattened `extra` map for vendor-specific properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterStatus {
    /// what state the printer is currently in
    pub state: State,
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
    pub extra: HashMap<String, Value>,
}

impl PartialEq for PrinterStatus {
    fn eq(&self, other: &Self) -> bool {
        // Helper for optional f64 comparisons with relative_eq
        let f64_opt_eq = |a: &Option<f64>, b: &Option<f64>| match (a, b) {
            (Some(v1), Some(v2)) => relative_eq!(v1, v2, epsilon = 1e-4),
            (None, None) => true,
            _ => false,
        };

        relative_eq!(self.bed_temp, other.bed_temp, epsilon = 1e-4)
            && f64_opt_eq(&self.target_bed_temp, &other.target_bed_temp)
            && relative_eq!(self.nozzle_temp, other.nozzle_temp, epsilon = 1e-4)
            && f64_opt_eq(&self.target_nozzle_temp, &other.target_nozzle_temp)
            && f64_opt_eq(&self.chamber_temp, &other.chamber_temp)
            && self.extra == other.extra
            && self.state == other.state
    }
}

impl PrinterStatus {
    /// Creates a new [`PrinterStatus`] by combining core temperature telemetry with a
    /// strongly-typed vendor extension struct.
    ///
    /// # Errors
    /// Returns a [`serde_json::Error`] if the provided `extra` extension struct fails
    /// to serialize into a valid JSON object map.
    pub fn with_extras<E: Extensions>(
        state: State,
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
            state,
            extra: extras,
        })
    }

    /// create a new printer status object the given arguments
    pub fn new(
        state: State,
        bed_temp: f64,
        target_bed_temp: Option<f64>,
        nozzle_temp: f64,
        target_nozzle_temp: Option<f64>,
        chamber_temp: Option<f64>,
    ) -> Self {
        Self {
            bed_temp,
            target_bed_temp,
            nozzle_temp,
            target_nozzle_temp,
            chamber_temp,
            state,
            extra: HashMap::new(),
        }
    }

    /// determine if the printer is reporting a ready state
    pub fn is_ready(&self) -> bool {
        self.state.is_ready()
    }

    /// Determines the discrete [`EventType`] by comparing the previous job status state
    /// with the current job status state (`self`).
    pub fn get_event_type(&self, last: &PrinterStatus) -> EventType {
        if last.state == self.state {
            return EventType::StatusUpdate;
        }

        match (&last.state, &self.state) {
            // Transition from inactive/ready to printing -> Job Started
            (State::Idle | State::Ready, State::Printing) => EventType::PrintStarted,

            // Transition from printing to paused -> Job Paused
            (State::Printing, State::Paused) => EventType::PrintPaused,

            // Transition from paused back to printing -> Job Resumed
            (State::Paused, State::Printing) => EventType::PrintResumed,

            // Transition from active states to finished or stopped -> Job Stopped/Completed
            (State::Printing | State::Paused, State::Finished | State::Stopped) => {
                EventType::PrintStopped
            }

            // Transition into an error state
            (_, State::Error) => EventType::Err,

            // Fallback for any other state modification
            _ => EventType::StatusUpdate,
        }
    }
}

/// Information tracking the progress and timing of an active print job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobStatus {
    /// Estimated time remaining until job completion, measured in seconds.
    pub time_remaining_secs: i64,
    /// Current progress of the print job represented as a percentage (0 to 100).
    pub progress: i64,

    /// Private catch-all for vendor-specific JSON fields, flattened during serialization.
    #[serde(flatten)]
    pub extras: HashMap<String, Value>,
}
impl JobStatus {
    /// construct a new [`JobStatus`] with the given extra vendor specific values.
    pub fn with_extras<E: Extensions>(
        time_remaining_secs: i64,
        progress: i64,
        extra: E,
    ) -> Result<Self, serde_json::Error> {
        let val = serde_json::to_value(extra)?;
        let mut extras: HashMap<String, Value> = serde_json::from_value(val)?;
        extras.retain(|_, v| !v.is_null());
        Ok(Self {
            time_remaining_secs,
            progress,
            extras,
        })
    }

    /// construct a new [`JobStatus`] without any vendor specific extras.
    pub fn new(time_remaining_secs: i64, progress: i64) -> Self {
        Self {
            time_remaining_secs,
            progress,
            extras: HashMap::new(),
        }
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
