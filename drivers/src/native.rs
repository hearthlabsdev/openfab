//! Module to provide a native driver runtime environment.

use std::collections::HashMap;
use std::sync::{Arc};
use tokio::sync::Mutex;

use crate::config::{DriverMeta, DeviceDriver};
use crate::errors::DriverError;
use crate::runtime::{DriverHandle, DriverRuntime};
use crate::config::DriverMetadata;

pub struct NativeRuntime {
    pub drivers: HashMap<String, DriverHandle>,
}

impl NativeRuntime {
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
        }
    }
}

impl Default for NativeRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl DriverRuntime for NativeRuntime {
    async fn load<D>(&mut self, driver: D) -> Result<(), DriverError>
    where
        D: DeviceDriver + Send + 'static,
    {
        let metadata = driver.metadata();
        let id = metadata.id().to_string();

        let driver: DriverHandle = Arc::new(Mutex::new(Box::new(driver)));

        self.drivers.insert(id, driver);

        Ok(())
    }

    async fn index_drivers(&self) -> Result<Vec<DriverMeta>, DriverError> {
        let mut metadata = Vec::new();
        for (_, driver) in self.drivers.iter() {
            let dr = driver.lock().await;
            metadata.push((*dr.metadata()).into());
            drop(dr);
        }
        Ok(metadata)
    }

    async fn unload(&mut self, id: &str) -> Result<(), DriverError> {
        self.drivers
            .remove(id)
            .ok_or(DriverError::DriverNotFound)?;

        Ok(())
    }

    fn get_driver(&self, id: &str) -> Result<DriverHandle, DriverError> {
        self.drivers
            .get(id)
            .cloned()
            .ok_or(DriverError::DriverNotFound)
    }
}