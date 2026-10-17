//! Driver orchestration: configuring, loading/unloading drivers,
//! and managing driver connections.

use crate::DeviceDriver;
use crate::config::{ConfigSchema, DriverMeta};
use crate::errors::DriverError;
use crate::events::Event;
use crate::files::FileEntry;

use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::broadcast::Receiver;

use async_trait::async_trait;
use serde_json::Value;

pub type DriverHandle = Arc<Mutex<Box<dyn DeviceDriver + Send + Sync>>>;

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

    async fn list_entries(
        &self,
        driver: &str,
        device: &str,
        dir: &str,
    ) -> Result<Vec<FileEntry>, DriverError> {
        Ok(self
            .get_driver(driver)?
            .lock()
            .await
            .list_entries(device, dir)
            .await?)
    }

    async fn shutdown(&mut self) -> Result<(), DriverError> {
        for driver_meta in self.index_drivers().await?.iter() {
            self.unload(&driver_meta.id).await?;
        }
        Ok(())
    }
}
