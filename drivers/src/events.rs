use crate::PrinterStatus;
use crate::errors::PrintError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintInfo {
    file: FileInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Hash, PartialEq, Eq)]
pub enum EventType {
    Connected,
    Disconnected,
    PrintStarted,
    PrintPaused,
    PrintResumed,
    PrintStopped,
    FileUploaded,
    /// used to indicate any event when registering a callback.
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Connected(String),
    Disconnected(String),
    PrintStarted(PrintInfo),
    PrintPaused(PrintInfo),
    PrintResumed(PrintInfo),
    PrintStopped(PrintInfo, Option<PrintError>),
    FileUploaded(FileInfo),
    StatusUpdate(PrinterStatus),
    Err(String),
}

pub struct EventSet {
    pub events: Vec<Event>,
}
pub trait EventHandler {
    type Status;
    fn process(&mut self, status: Self::Status) -> Vec<Event>;
}

impl<E: EventHandler> From<(&mut E, E::Status)> for EventSet {
    fn from((handler, status): (&mut E, E::Status)) -> EventSet {
        EventSet { events: handler.process(status) }
    }
}
