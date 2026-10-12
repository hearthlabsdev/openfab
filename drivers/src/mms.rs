//! # Multi-Material System (MMS) Subsystem
//!
//! Provides brand-agnostic data structures for tracking multi-spool switching hubs,
//! unifying hardware concepts like the Prusa MMU, Bambu AMS, Creality CFS, and Anycubic ACE Pro.

use serde::{Deserialize, Serialize};

/// Represents the global state of a Multi-Material System hub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiMaterialExtension {
    pub state: String,
    /// The currently active slot index being fed into the extruder, if any.
    pub active_slot: Option<u8>,
    pub previous_slot: Option<u8>,
    pub target_slot: Option<u8>,
    /// Telemetry and configuration for each individual filament slot/bay.
    pub slots: Vec<SlotStatus>,
    /// Optional environmental telemetry from the hub (e.g., Bambu AMS dry box humidity)
    pub environmental: Option<MmsEnvironment>,
}

/// Telemetry for an individual filament slot within an MMS hub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotStatus {
    /// The zero-indexed slot number (e.g., 0, 1, 2, 3).
    pub index: u8,
    /// Whether filament is currently detected loaded in this slot.
    pub loaded: bool,
    /// The declared material type (e.g., "PLA", "PETG", "ABS").
    pub filament_type: Option<String>,
    /// Associated color hex code or identifier, if reported by the hub.
    pub color_hex: Option<String>,
    pub remaining_percentage: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmsEnvironment {
    pub humidity_percent: Option<u8>,
    pub temperature_celsius: Option<f32>,
}