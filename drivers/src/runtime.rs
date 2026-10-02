//! Driver orchestration: configuring, loading/unloading drivers,
//! and managing driver connections.

use crate::config::{ConfigSchema, DriverMeta, DeviceDriver};
use crate::errors::DriverError;
use crate::config::DriverMetadata;

use async_trait::async_trait;
use serde_json::Value;
use std::sync::{Arc};
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

    async fn config_schema(&self, id: &str) -> Result<ConfigSchema, DriverError> {
        let driver = self.get_driver(id)?;
        let driver = driver
            .lock().await;

        Ok(driver.metadata().config_schema())
    }

    async fn add_device(
        &self,
        id: &str,
        config: Value,
    ) -> Result<(), DriverError> {
        let driver = self.get_driver(id)?;

        let mut driver = driver
            .lock().await;
        Ok(driver.connect(config).await?)
    }
}
