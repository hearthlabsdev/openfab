//! # Event System and Handler Framework
//!
//! This module defines the core, vendor-agnostic event model and handling traits for the driver SDK.
//!
//! Events represent state transitions, lifecycle changes, and telemetry updates produced by hardware
//! backends (e.g., 3D printers, CNC routers, vinyl cutters). To maintain a clean, non-generic pipeline,
//! events carry non-generic payload structures like [`PrinterStatus`], allowing downstream event loops,
//! handlers, and UI applications to process telemetry without generic type contagion.

use crate::DriverError;
use crate::PrinterStatus;
use crate::errors::PrintError;
use crate::errors::StatusErr;
use crate::status::JobStatus;
use serde::{Deserialize, Serialize};

/// Metadata regarding a specific print job execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrintInfo {
    /// Information about the print file associated with the job.
    pub file: FileInfo,
}

impl PrintInfo {
    /// Creates a new [`PrintInfo`] instance for a given file.
    pub fn new(file_name: impl Into<String>) -> Self {
        Self {
            file: FileInfo {
                name: file_name.into(),
            },
        }
    }
}

/// Metadata describing a file managed or processed by a device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileInfo {
    /// The filename or path identifier on the storage media.
    pub name: String,
}

impl FileInfo {
    /// Creates a new [`FileInfo`] instance with the specified name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Category markers representing distinct event categories.
///
/// Used for filtering event subscriptions, registering callbacks, or routing events.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum EventType {
    /// The driver established a connection with the hardware backend.
    Connected,
    /// The driver lost connection or disconnected from the hardware backend.
    Disconnected,
    /// A print job execution started.
    PrintStarted,
    /// An active print job was paused.
    PrintPaused,
    /// A paused print job was resumed.
    PrintResumed,
    /// A print job execution was stopped or cancelled.
    PrintStopped,
    /// A new file was uploaded to the hardware backend.
    FileUploaded,
    /// A telemetry or status snapshot update was received.
    StatusUpdate,
    /// An error event occurred within the driver or backend connection.
    Err,
    /// A wildcard marker used to indicate subscription to any event category when registering callbacks.
    Any,
    JobUpdate,
}

/// Discrete lifecycle and status events emitted by driver backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Connection established. Carries the target endpoint or device identifier string.
    Connected(String),
    /// Connection lost or closed. Carries the endpoint or device identifier string.
    Disconnected(String),
    /// A print job has begun.
    PrintStarted(JobStatus),
    /// Job Status update.
    JobUpdate(JobStatus),
    /// A print job was paused.
    PrintPaused(JobStatus),
    /// A print job was resumed.
    PrintResumed(JobStatus),
    /// A print job was stopped, optionally carrying error information if aborted due to a failure.
    PrintStopped(JobStatus, Option<PrintError>),
    /// A file was successfully uploaded to the device.
    FileUploaded(FileInfo),
    /// A periodic status snapshot was received from the hardware.
    StatusUpdate(PrinterStatus),
    //CameraUpdate(CameraStatus),
    /// A driver-level or backend error occurred.
    Err(String),
}

impl Event {
    /// Returns the corresponding [`EventType`] category for this event instance.
    pub fn event_type(&self) -> EventType {
        match self {
            Event::Connected(_) => EventType::Connected,
            Event::Disconnected(_) => EventType::Disconnected,
            Event::PrintStarted(_) => EventType::PrintStarted,
            Event::PrintPaused(_) => EventType::PrintPaused,
            Event::PrintResumed(_) => EventType::PrintResumed,
            Event::PrintStopped(_, _) => EventType::PrintStopped,
            Event::FileUploaded(_) => EventType::FileUploaded,
            Event::StatusUpdate(_) => EventType::StatusUpdate,
            Event::JobUpdate(_) => EventType::JobUpdate,
            Event::Err(_) => EventType::Err,
        }
    }
}

/// A container wrapping a collection of events produced during a single processing cycle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventSet {
    /// The list of events generated in this cycle.
    pub events: Vec<Event>,
}

impl EventSet {
    /// Creates an empty [`EventSet`].
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Creates an [`EventSet`] containing a single event.
    pub fn single(event: Event) -> Self {
        Self {
            events: vec![event],
        }
    }

    /// Returns `true` if the set contains no events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Returns the number of events contained in the set.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Retains only the events matching the specified [`EventType`].
    pub fn filter_by_type(&self, event_type: EventType) -> impl Iterator<Item = &Event> {
        self.events
            .iter()
            .filter(move |e| event_type == EventType::Any || e.event_type() == event_type)
    }
}

impl IntoIterator for EventSet {
    type Item = Event;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}

impl From<Vec<Event>> for EventSet {
    fn from(events: Vec<Event>) -> Self {
        Self { events }
    }
}

/// A trait for processing raw status updates or hardware payloads into structured driver events.
///
/// Implementors convert driver-specific status payloads (`Status`) into one or more [`Event`]
/// instances wrapped in an [`EventSet`].
pub trait EventHandler {
    /// The input status payload type consumed by this handler.
    type Status;

    /// Processes an incoming raw status snapshot and yields a list of events generated by state changes.
    fn process(&mut self, status: Self::Status) -> Result<Vec<Event>, StatusErr>;
}

impl<E: EventHandler> TryFrom<(&mut E, E::Status)> for EventSet {
    type Error = StatusErr;
    /// Converts a tuple of an [`EventHandler`] mutable reference and its associated `Status`
    /// directly into an [`EventSet`].
    fn try_from((handler, status): (&mut E, E::Status)) -> Result<EventSet, StatusErr> {
        Ok(EventSet {
            events: handler.process(status)?,
        })
    }
}

pub enum CameraEvent {
    Image(Vec<u8>),
    Error(DriverError),
}
