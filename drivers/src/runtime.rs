//! Driver orchestration: configuring, loading/unloading drivers,
//! and managing driver connections.

use crate::Callback;
use crate::DeviceDriver;
use crate::config::{ConfigSchema, DriverMeta};
use crate::errors::DriverError;
use crate::events::{Event, EventType};
use tokio::sync::broadcast::Receiver;

use crate::DeviceCallback;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type DriverHandle = Arc<Mutex<Box<dyn DeviceDriver + Send>>>;

#[async_trait]
pub trait DriverRuntime: Send + Sync {
    async fn load<D>(&mut self, driver: D) -> Result<(), DriverError>
    where
        D: DeviceDriver + Send + 'static;

    async fn unload(&mut self, id: &str) -> Result<(), DriverError>;

    fn get_driver(&self, id: &str) -> Result<DriverHandle, DriverError>;

    async fn index_drivers(&self) -> Result<Vec<DriverMeta>, DriverError>;

    async fn subscribe(&self, driver: &str, device: &str) -> Result<Receiver<Event>, DriverError> {
        Ok(self.get_driver(driver)?.lock().await.subscribe(device)?)
    }
    async fn config_schema(&self, id: &str) -> Result<ConfigSchema, DriverError> {
        let driver = self.get_driver(id)?;
        let driver = driver.lock().await;

        Ok(driver.metadata().config_schema().clone())
    }

    async fn add_device(&self, id: &str, config: Value) -> Result<(), DriverError> {
        let driver = self.get_driver(id)?;

        let mut driver = driver.lock().await;
        Ok(driver.connect(config).await?)
    }

    /*
    async fn register_callback(&mut self, driver: &str, device: &str, event_type: EventType, callback: impl DeviceCallback) -> Result<(), DriverError> {

        let boxed_cb = Box::new(move |event, device| {
            callback.call(event, device)
        });

        self.get_driver(driver)?.lock().await.register_callback(device, event_type, boxed_cb).await
    }*/

    async fn register_handler(
        &mut self,
        driver: &str,
        device: &str,
        handler: impl DeviceCallback,
    ) -> Result<(), DriverError> {
        self.get_driver(driver)?
            .lock()
            .await
            .register_handler(
                device,
                Box::new(move |stream| Box::pin(handler.call(stream))),
            )
            .await
    }
}
