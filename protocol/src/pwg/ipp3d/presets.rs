//! # IPP 3D Presets
//!
//! This module provides **compile-time–embedded presets** for common 3D printers
//! and materials, intended to complement the strongly-typed IPP 3D attribute
//! structures defined in the parent module.
//!
//! Presets are distributed as a JSON file that is included at compile time using
//! `include_str!`, allowing:
//!
//! * Zero runtime file I/O
//! * Deterministic, reproducible builds
//! * Easy downstream consumption (CLI tools, services, GUIs)
//!
//! The data in this module is **descriptive**, not normative: it represents
//! reasonable defaults for well-known printers and materials, and may be
//! customized or overridden by applications as needed.
//!
//! ## Design Goals
//!
//! * **Separation of data and protocol** — presets are plain data, independent
//!   of IPP wire encoding.
//! * **Interoperability** — preset types map cleanly onto IPP 3D structs such as
//!   [`Material`] and [`BuildVolume`].
//! * **Extensibility** — additional printers, materials, or vendor-specific
//!   fields can be added to the JSON without changing calling code.
//!
//! ## Preset Structure
//!
//! The embedded JSON file defines two top-level collections:
//!
//! * **materials** — reusable material definitions
//! * **printers** — printer models referencing supported materials by name
//!
//! This allows materials to be defined once and reused across multiple printer
//! presets.
//!
//! ## Usage
//!
//! ```rust
//! use crate::ipp3d::presets::load_presets;
//!
//! let presets = load_presets();
//!
//! for printer in presets.printers {
//!     println!("Printer model: {}", printer.model);
//! }
//! ```
//!
//! ## Notes
//!
//! * Presets are intended as **starting points**, not authoritative device
//!   capability descriptions.
//! * Applications SHOULD validate presets against live
//!   `Get-Printer-Attributes` responses when available.
//! * Material names referenced by `PrinterPreset::supported_materials` are
//!   expected to match entries in `PresetFile::materials`.
//!
//! ## Future Extensions
//!
//! * Vendor-specific capability flags
//! * Region- or market-specific defaults
//! * Localization of human-readable fields
//! * Conversion helpers from presets to `PrinterCapabilities`

use serde_derive::{Deserialize, Serialize};

use super::*;

/// Defines a preset 3D printer model.
///
/// This type represents a well-known printer configuration with a fixed build
/// volume and a set of supported materials. It is typically sourced from the
/// embedded preset JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterPreset {
    /// Human-readable model identifier of the printer.
    ///
    /// Example: `"Prusa MK4"` or `"Ultimaker S5"`.
    pub model: String,

    /// Physical build volume of the printer.
    pub build_volume: BuildVolume,

    /// Names of materials supported by this printer.
    ///
    /// Each entry should correspond to a `Material::name` defined in the
    /// `PresetFile::materials` list.
    pub supported_materials: Vec<String>,
}

/// Represents the parsed contents of the embedded printer preset file.
///
/// This structure is the root of the preset JSON schema and groups reusable
/// materials with printer models that reference them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetFile {
    /// List of reusable material definitions.
    pub materials: Vec<Material>,

    /// List of predefined printer models.
    pub printers: Vec<PrinterPreset>,
}

/// Embedded JSON containing default printer and material presets.
///
/// This file is included at compile time to avoid runtime file access and
/// to ensure consistent, reproducible preset data.
static PRESET_JSON: &str = include_str!("printer_presets.json");

/// Loads the embedded printer and material presets.
///
/// # Panics
///
/// Panics if the embedded `printer_presets.json` file cannot be parsed. This is
/// considered a programmer error and should only occur if the preset file is
/// invalid or corrupted during development.
pub fn load_presets() -> PresetFile {
    serde_json::from_str(PRESET_JSON).expect("Failed to parse printer_presets.json")
}
