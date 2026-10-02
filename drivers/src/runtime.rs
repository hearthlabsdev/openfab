//! This module handles driver orchestration including how to configure drivers, load, unload drivers, and manage driver connections.
use crate::errors::{DriverError};
use crate::config::{DeviceDriver, ConfigSchema};
use async_trait::async_trait;

#[async_trait]
pub trait DriverRuntime {
    async fn load<D: DeviceDriver + 'static>(&mut self, driver: D) -> Result<(), DriverError>;
    async fn unload(&mut self, id: &str) -> Result<(), DriverError>;
    async fn config_schema(&self, id: &str) -> Result<ConfigSchema, DriverError>;
}