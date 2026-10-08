// for defining traits related to firmware drivers (marlin, klipper)
pub mod firmware;
pub mod capabilities;
pub mod formats;
pub mod config;
pub mod runtime;
pub mod errors;
pub mod native;
pub mod utils;
pub mod status;
pub mod events;
use crate::events::*;
use crate::errors::{DriverError, PrintError};
use crate::status::PrinterStatus;
use crate::config::DriverMeta;

use serde::{Serialize, Deserialize};

use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[async_trait::async_trait]
pub trait DeviceDriver: Send + Sync {
    fn metadata(&self) -> DriverMeta;
    // Requires &self or &mut self to be callable on a `dyn DeviceDriver`
    async fn connect(&mut self, config: serde_json::Value) -> Result<(), DriverError>;
    async fn disconnect(&mut self) -> Result<(), DriverError>;
    async fn upload_file(&mut self) -> Result<(), DriverError>;
    async fn get_status(&self, device: &str) -> Result<(), DriverError>;

    /// Register an event callback handler
    async fn register_callback(
        &mut self,
        event: EventType,
        callback: Box<
            dyn FnMut(String, Event) -> BoxFuture<'static, Result<Option<PrinterStatus>, DriverError>>
                + Send
                + 'static,
        >,
    ) -> Result<(), DriverError>;
}