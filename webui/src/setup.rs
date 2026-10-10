/// Utility module for setting up server runtime.
use crate::devices::{DeviceConfigORM, DeviceORM};
use crate::errors::OpenFabErr;

use std::collections::HashMap;

use openfab_drivers::DeviceDriver;
use openfab_drivers::config::{ConfigSchema, DriverMeta};
use openfab_drivers::errors::DriverError;
use openfab_drivers::events::{Event, EventType};
use openfab_drivers::native::NativeRuntime;
use openfab_drivers::runtime::DriverHandle;
use openfab_drivers::runtime::DriverRuntime as Runtime;
use serde_json::Value;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

use ormlite::Model;
use ormlite::postgres::PgPool;
use uuid::Uuid;

pub type NativeManager = RuntimeManager<NativeRuntime>;

pub struct RuntimeManager<R: Runtime + 'static> {
    runtime: R,
    senders: HashMap<Uuid, Sender<Event>>,
}

impl<R: Runtime + 'static> RuntimeManager<R> {
    pub fn new(rt: R) -> Self {
        Self {
            runtime: rt,
            senders: HashMap::new(),
        }
    }
    pub fn subscribe(&self, device: Uuid) -> Result<Receiver<Event>, OpenFabErr> {
        Ok(self
            .senders
            .get(&device)
            .ok_or(OpenFabErr::MissingSender(device.clone()))?
            .subscribe())
    }
    pub async fn load_driver<D: DeviceDriver + 'static>(
        &mut self,
        driver: D,
    ) -> Result<(), OpenFabErr> {
        Ok(self.runtime.load(driver).await?)
    }
    pub async fn load_devices(&mut self, pool: &PgPool) -> Result<(), OpenFabErr> {
        let mut conn = pool.acquire().await?;

        for device in DeviceORM::select().fetch_all(&mut *conn).await? {
            let (sender, mut rx1) = broadcast::channel(100);
            let sender2 = sender.clone();

            let configs = DeviceConfigORM::select()
                .where_("device = ?")
                .bind(device.uid)
                .fetch_all(&mut *conn)
                .await?
                .into_iter()
                .map(|v| (v.key, v.value))
                .collect::<HashMap<String, String>>();

            let schema = self.runtime.config_schema(&device.driver_id).await?;
            let device_config = openfab_drivers::utils::config_to_json(&schema, &configs)?;
            self.senders.insert(device.uid, sender);
            self.runtime
                .add_device(&device.driver_id, device_config)
                .await?;
            self.runtime
                .register_callback(
                    &device.driver_id,
                    &device.serial,
                    EventType::Any,
                    move |device, event| {
                        let mut cbsender = sender2.clone();
                        async move {
                            cbsender.send(event).unwrap();
                            Ok(None)
                        }
                    },
                )
                .await?;
        }
        Ok(())
    }

    pub async fn config_schema(&self, id: &str) -> Result<ConfigSchema, DriverError> {
        self.runtime.config_schema(id).await
    }

    pub async fn add_device(&self, id: &str, config: Value) -> Result<(), DriverError> {
        self.runtime.add_device(id, config).await
    }

    pub async fn index_drivers(&self) -> Result<Vec<DriverMeta>, DriverError> {
        self.runtime.index_drivers().await
    }
}
/*
#[async_trait::async_trait]
impl<R: Runtime + 'static> Runtime for RuntimeManager<R> {
    async fn load<D>(&mut self, driver: D) -> Result<(), DriverError>
    where
        D: DeviceDriver + Send + 'static {
            self.runtime.load(driver).await
        }

    async fn unload(&mut self, id: &str) -> Result<(), DriverError> {
        self.runtime.unload(id).await
    }

    fn get_driver(&self, id: &str) -> Result<DriverHandle, DriverError> {
        self.runtime.get_driver(id)
    }

    async fn index_drivers(&self) -> Result<Vec<DriverMeta>, DriverError> {
        self.runtime.index_drivers().await
    }
}*/
