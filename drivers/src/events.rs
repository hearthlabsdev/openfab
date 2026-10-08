use crate::errors::PrintError;
use crate::PrinterStatus;
use serde::{Serialize, Deserialize};

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
}
