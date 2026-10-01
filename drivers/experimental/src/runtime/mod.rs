use wasmi::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::driver::DriverState;
use crate::errors::DriverErr;

// =========================
// Per-driver instance
// =========================
pub struct DriverInstance {
    pub store: Store<HostState>,
    pub instance: wasmi::Instance,
}

// =========================
// Host state (per Store)
// =========================
pub struct HostState {
    pub driver_path: PathBuf,
    pub state: DriverState,
}

// =========================
// Runtime (global)
// =========================
pub struct Runtime {
    engine: Engine,
    modules: HashMap<String, Module>, // cache compiled modules

}

impl Runtime {
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
            modules: HashMap::new(),
        }
    }

    /// Load + cache a module (compile once)
    pub fn register_module(
        &mut self,
        name: impl Into<String>,
        wasm: &[u8],
    ) -> Result<(), DriverErr> {
        let module = Module::new(&self.engine, wasm)?;
        self.modules.insert(name.into(), module);
        Ok(())
    }

    /// Instantiate a driver (creates isolated Store)
    pub fn instantiate_driver(
        &self,
        module_name: &str,
        driver_path: PathBuf,
        driver_state: DriverState,
    ) -> Result<DriverInstance, DriverErr> {
        let module = self
            .modules
            .get(module_name)
            .ok_or(DriverErr::ModuleNotFound)?;

        // Each driver gets its own host state
        let host_state = HostState {
            driver_path,
            state: driver_state,
        };

        // IMPORTANT: use the SAME engine
        let mut store = Store::new(&self.engine, host_state);

        let mut linker = Linker::new(&self.engine);

        // Example host function (you’ll expand this later)
        linker.func_wrap(
            "env",
            "log",
            |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| {
                // You can access driver-specific state here
                let _state = caller.data_mut();
                println!("Driver log called (ptr={}, len={})", ptr, len);
            },
        )?;

        let instance = linker.instantiate_and_start(&mut store, module)?;

        Ok(DriverInstance { store, instance })
    }
}