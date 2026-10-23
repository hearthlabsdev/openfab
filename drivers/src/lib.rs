#![deny(unused)]
//#![deny(missing_docs)]
//! # Fabrication Machine Driver SDK
//!
//! This crate provides a unified, vendor-agnostic SDK and interface layer for interacting
//! with diverse manufacturing and fabrication hardware (such as 3D printers, laser cutters,
//! and CNC machines).
//!
//! ## Core Architecture
//! The SDK decouples hardware communication from application logic by using:
//! - **Standardized Core Telemetry**: Agnostic structures like [`PrinterStatus`] and [`Event`].
//! - **Driver Abstraction**: The [`DeviceDriver`] async trait, which standardizes connection handling,
//!   subscriptions, and event streaming across vastly different vendor APIs.
//! - **Encapsulated Extensions**: Type-safe extension layers for brand-specific features without
//!   contaminating core event loops.

//! # Example: Implementing `DeviceDriver` for a Bambu Lab Backend
//!
//! Below is an example of how a vendor driver (e.g., Bambu Lab) implements the unified
//! [`DeviceDriver`] trait, managing background connection workers, broadcasting status events,
//! and handling client registrations.
//!
//! -- Example Implementation Block --
//! ```rust
//! #[async_trait::async_trait]
//! impl DeviceDriver for BambuDriver {
//!     fn metadata(&self) -> DriverMeta {
//!         Self::metadata()
//!     }
//!
//!     async fn connect(&mut self, config: serde_json::Value) -> Result<(), DriverError> {
//!         let target: ClientTarget = serde_json::from_value(config)?;
//!         let serial = target.serial.clone();
//!         let resolved = target.resolve();
//!         let client = Arc::new(LanMqttClient::new(resolved));
//!         let worker_client = client.clone();
//!         let (sender, recv) = channel(20);
//!         let sender2 = sender.clone();
//!
//!         match worker_client.fetch_async().await {
//!             Ok(event) => {
//!                 let status = BambuStatus::from_state(event.get());
//!                 let printer_status = try_from_device_status(status)
//!                     .map_err(|e| DriverError::Custom(e.to_string()))?;
//!                     sender2.send(Event::Connected(serial));
//!                     sender2.send(Event::StatusUpdate(printer_status))
//!                     .map_err(|_| DriverError::Custom("failed to send initial status".into()))?;
//!             }
//!             Err(e) => {
//!                 return Err(DriverError::Custom(e.to_string()));
//!             }
//!         }
//!
//!         let handle = tokio::task::spawn(async move {
//!             loop {
//!                 match worker_client.fetch_async().await {
//!                     Ok(event) => {
//!                         let status = BambuStatus::from_state(event.get());
//!                         if let Ok(printer_status) = try_from_device_status(status) {
//!                             let _ = sender2.send(Event::StatusUpdate(printer_status));
//!                         }
//!                     }
//!                     Err(e) => {
//!                         let _ = sender2.send(Event::Err(e.to_string()));
//!                     }
//!                 }
//!                 tokio::time::sleep(Duration::from_millis(1000)).await;
//!             }
//!         });
//!
//!         let lan_client = BambuClient {
//!             worker: handle,
//!             client,
//!             sender,
//!             recv,
//!             handler: None,
//!         };
//!         
//!         self.clients.insert(serial, lan_client);
//!         Ok(())
//!     }
//!
//!     async fn disconnect(&mut self, device: &str) -> Result<(), DriverError> {
//!         if let Some(client) = self.clients.remove(device) {
//!             client.worker.abort();
//!         }
//!         Ok(())
//!     }
//!
//!     async fn upload_file(&mut self, _file_path: &str) -> Result<(), DriverError> {
//!         unimplemented!("File upload for Bambu driver pending implementation");
//!     }
//!
//!     async fn get_status(&self, device: &str) -> Result<PrinterStatus, DriverError> {
//!         let client = self.clients.get(device)
//!             .ok_or_else(|| DriverError::DeviceNotFound(device.into()))?;
//!         // Fetch or reconstruct latest status...
//!         unimplemented!()
//!     }
//!
//!     fn subscribe(&self, device: &str) -> Result<Receiver<Event>, DriverError> {
//!         let client = self.clients.get(device)
//!             .ok_or_else(|| DriverError::DeviceNotFound(device.into()))?;
//!         Ok(client.sender.subscribe())
//!     }
//!
//!     async fn register_handler(
//!         &mut self,
//!         device: &str,
//!         callback: Callback,
//!     ) -> Result<(), DriverError> {
//!         let client = self.clients.get_mut(device)
//!             .ok_or_else(|| DriverError::DeviceNotFound(device.into()))?;
//!         let recv = client.sender.subscribe();
//!         let handle = tokio::spawn(async move {
//!             callback(recv).await
//!         });
//!         client.handler = Some(handle);
//!         Ok(())
//!     }
//! }
//!```

// Subsystem module declarations
pub mod capabilities;
pub mod config;
pub mod errors;
pub mod events;
pub mod files;
pub mod firmware;
pub mod formats;
pub mod mms;
pub mod native;
pub mod runtime;
pub mod status;
pub mod utils;

use crate::config::DriverMeta;
use crate::errors::DriverError;
use crate::events::*;
use crate::files::FileEntry;
use crate::status::PrinterStatus;

use tokio::sync::broadcast::Receiver;
use tokio::sync::watch::Receiver as Watcher;
use tokio::io::AsyncRead;

use serde::{Deserialize, Serialize};

use std::path::Path;
use std::pin::Pin;
use std::io::Cursor;

/// Boxed async reader type used for object-safe upload_file signature.
pub type BoxAsyncRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;

/// A unique identifier linking a specific device instance to an event category subscription.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CallbackId {
    /// Unique identifier of the target hardware device.
    pub device: String,
    /// The event category being filtered or targeted.
    pub event: EventType,
}

/// The core asynchronous trait implemented by all hardware driver backends
/// (e.g., Prusa Link, Klipper, Bambu Lab).
///
/// This trait establishes a unified contract for establishing connections, managing file uploads,
/// querying device status, and subscribing to live event broadcasts.
#[async_trait::async_trait]
pub trait DeviceDriver: Send + Sync {
    /// Returns metadata describing the driver and its configuration schema.
    fn metadata(&self) -> DriverMeta;

    /// Establishes an active connection with a hardware endpoint using configuration parameters.
    async fn connect(&mut self, config: serde_json::Value) -> Result<(), DriverError>;

    /// Gracefully disconnects and tears down workers for the specified device identifier.
    async fn disconnect(&mut self, device: &str) -> Result<(), DriverError>;

    async fn shutdown(&self) -> Result<(), DriverError>;

    /// Uploads a print or fabrication file to the device storage.
    async fn upload_file(&mut self, device: &str, path: &Path, reader: BoxAsyncRead) -> Result<(), DriverError>;
    async fn download_bytes(&self, device: &str, path: &Path) -> Result<Cursor<Vec<u8>>, DriverError>;
    async fn upload_bytes(&mut self, device: &str, path: &Path, file: Vec<u8>) -> Result<usize, DriverError> {
        let len = file.len();
        self.upload_file(device, path, Box::pin(std::io::Cursor::new(file))).await?;
        Ok(len)
    }

    /// Retrieves an immediate snapshot of the current printer status.
    async fn get_status(&self, device: &str) -> Result<PrinterStatus, DriverError>;

    /// Returns a broadcast receiver stream for listening to live driver events.
    fn subscribe(&self, device: &str) -> Result<Receiver<Event>, DriverError>;

    async fn list_entries(&self, device: &str, dir: &str) -> Result<Vec<FileEntry>, DriverError>;

    fn subscribe_camera(&self, _device: &str) -> Result<Option<Watcher<CameraEvent>>, DriverError> {
        Ok(None)
    }

    async fn start_print(&self, _device: &str, _path: &Path) -> Result<(), DriverError> {
        Ok(())
    }
    async fn pause_print(&self, _device: &str) -> Result<(), DriverError> {
        Ok(())
    }

    async fn resume_print(&self, _device: &str) -> Result<(), DriverError> {
        Ok(())
    }
    async fn stop_print(&self, _device: &str) -> Result<(), DriverError> {
        Ok(())
    }
}
