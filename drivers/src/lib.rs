// for defining traits related to firmware drivers (marlin, klipper)
pub mod capabilities;
pub mod config;
pub mod errors;
pub mod events;
pub mod firmware;
pub mod formats;
pub mod native;
pub mod runtime;
pub mod status;
pub mod utils;
use crate::config::DriverMeta;
use crate::errors::{DriverError, PrintError};
use crate::events::*;
use crate::status::PrinterStatus;

use serde::{Deserialize, Serialize};

use std::future::Future;
use std::pin::Pin;
use tokio::sync::broadcast::Receiver;

pub trait DeviceCallback: Send + Sync + 'static {
    fn call(
        self,
        stream: Receiver<Event>,
    ) -> impl Future<Output = Result<Option<PrinterStatus>, DriverError>> + Send + 'static;
}
impl<F, Fut> DeviceCallback for F
where
    F: FnOnce(Receiver<Event>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Option<PrinterStatus>, DriverError>> + Send + 'static,
{
    fn call(
        self,
        stream: Receiver<Event>,
    ) -> BoxFuture<'static, Result<Option<PrinterStatus>, DriverError>> {
        Box::pin((self)(stream))
    }
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type Callback = Box<
    dyn FnOnce(Receiver<Event>) -> BoxFuture<'static, Result<Option<PrinterStatus>, DriverError>>
        + Send
        + Sync
        + 'static,
>;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CallbackId {
    device: String,
    event: EventType,
}

#[async_trait::async_trait]
pub trait DeviceDriver: Send + Sync {
    fn metadata(&self) -> DriverMeta;
    // Requires &self or &mut self to be callable on a `dyn DeviceDriver`
    async fn connect(&mut self, config: serde_json::Value) -> Result<(), DriverError>;
    async fn disconnect(&mut self, device: &str) -> Result<(), DriverError>;
    async fn upload_file(&mut self) -> Result<(), DriverError>;
    async fn get_status(&self, device: &str) -> Result<PrinterStatus, DriverError>;
    fn subscribe(&self, device: &str) -> Result<Receiver<Event>, DriverError>;
    /*
    /// Register an event callback handler
    async fn register_callback(
        &mut self,
        device: &str,
        event: EventType,
        callback: Callback,
    ) -> Result<(), DriverError>;*/

    async fn register_handler(
        &mut self,
        device: &str,
        handler: Callback,
    ) -> Result<(), DriverError>;
}
