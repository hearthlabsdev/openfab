//!
//! OpenFab Driver Runtime Utility Crate.
//!
//! Implementing a Driver:
//! 
//! ```wit
//! 
//!
//! ```

#[cfg(feature = "serialport")]
pub mod serial;

pub mod errors;
pub mod runtime;
pub mod gpio;
pub mod driver;
pub mod devices;

#[cfg(feature = "serialport")]
wit_bindgen::generate!({
    path: "wit",
    world: "drivers"
});