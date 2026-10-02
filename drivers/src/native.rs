//! Module to provide a native driver runtime environment.
use std::collections::HashMap;
use crate::config::{DeviceDriver, ConfigSchema};
use crate::runtime::DriverRuntime;
use crate::errors::DriverError;

pub struct NativeRuntime {
    pub drivers: HashMap<String, Box<dyn DeviceDriver>>,
}

#[async_trait::async_trait]
impl DriverRuntime for NativeRuntime {
    async fn load<D: DeviceDriver + 'static>(&mut self, driver: D) -> Result<(), DriverError> {
        let metadata = driver.metadata();
        self.drivers.insert(metadata.id().to_string(), Box::new(driver));
        Ok(())
    }

    async fn unload(&mut self, id: &str) -> Result<(), DriverError> {
        self.drivers.remove(id);
        Ok(())
    }

    async fn config_schema(&self, id: &str) -> Result<ConfigSchema, DriverError> {
        Ok(self.drivers.get(id).ok_or(DriverError::DriverNotFound)?.metadata().config_schema())
    }
}