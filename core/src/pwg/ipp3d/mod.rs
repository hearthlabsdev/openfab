//! # IPP 3D Structured Attributes
//!
//! This module provides **strongly-typed Rust representations** of IPP 3D Printing
//! attributes, as defined in *PWG 5100.21-2019 — IPP 3D Printing Extensions*.
//!
//! The goal of this module is to model IPP 3D concepts in a way that is:
//!
//! * **Type-safe** — Rust structs reflect the structure of IPP collections
//! * **Protocol-correct** — values map cleanly to `IppValue` variants
//! * **Composable** — nested collections and arrays are first-class
//! * **Deterministic** — `BTreeMap` is used for stable attribute ordering
//!
//! All public structs expose a `to_ipp()` method that converts them into
//! `IppValue::Collection`, the canonical representation for structured IPP
//! attributes.
//!
//! ## Covered Concepts
//!
//! * **Material** — printable material definitions
//! * **BuildVolume** — physical constraints of a 3D printer
//! * **JobTicket** — per-job print configuration
//! * **PrinterCapabilities** — device-reported supported features
//!
//! ## Notes on IPP Semantics
//!
//! * Units are preserved exactly as defined by the IPP 3D specification
//!   (e.g., µm for resolution, mm for dimensions).
//! * Optional fields are only emitted if present, following IPP best practices.
//! * Identifiers and protocol-facing values use `TextWithoutLanguage` semantics
//!   (represented here via `String` → `IppValue`).
//!
//! ## References
//!
//! * PWG 5100.21-2019 — IPP 3D Printing Extensions
//! * RFC 8010, RFC 8011 — IPP Core Specifications
//!
//! Please see [IPP3D](https://ftp.pwg.org/pub/pwg/candidates/cs-ipp3d11-20190329-5100.21.pdf)

/// defines extension attributes
pub mod attributes;

/// represents a 3D collection (may replace with proper crate later).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Vec3D {
    /// x aspect of the 3D point/scalar. 
    pub x: i32,
    /// y aspect of the 3D point/scalar.
    pub y: i32,
    /// z aspect (height) of the 3D point/scalar.
    pub z: i32,
}

use std::collections::BTreeMap;

use ipp::prelude::IppValue;
use crate::ipputils::{AttrMap, AttributeGroup, IntegerRange};

pub mod presets;

/// Top-level constants for IPP 3D Printing "materials-col" attribute and its members.
pub mod tags {
    /// Top-level *Job Template* collection attribute.
    pub const MATERIALS_COL: &str = "materials-col";

    /// Human-readable, localized name of the material (type: name(MAX), REQUIRED)
    pub const MATERIAL_NAME: &str = "material-name";

    /// Unlocalized key / identifier for the material (type: keyword, REQUIRED)
    pub const MATERIAL_KEY: &str = "material-key";

    /// Material type (type2 keyword | name(MAX), REQUIRED)
    pub const MATERIAL_TYPE: &str = "material-type";

    /// Fill density (0–100) (type: integer(0:100), REQUIRED)
    pub const MATERIAL_FILL_DENSITY: &str = "material-fill-density";

    /// Material purpose(s) (type: 1setOf type2 keyword, REQUIRED)
    pub const MATERIAL_PURPOSE: &str = "material-purpose";

    /// Estimated or actual material amount (integer(0:MAX), OPTIONAL)
    pub const MATERIAL_AMOUNT: &str = "material-amount";

    /// Units for material amount (type2 keyword, OPTIONAL)
    pub const MATERIAL_AMOUNT_UNITS: &str = "material-amount-units";

    /// Material color (type2 keyword, OPTIONAL)
    pub const MATERIAL_COLOR: &str = "material-color";

    /// Filament diameter in nanometers (integer(0:MAX), OPTIONAL)
    pub const MATERIAL_DIAMETER: &str = "material-diameter";

    /// Tolerance for filament diameter in nanometers (integer(0:MAX), OPTIONAL)
    pub const MATERIAL_DIAMETER_TOLERANCE: &str = "material-diameter-tolerance";

    /// Extruder nozzle diameter (integer(0:MAX), OPTIONAL)
    pub const MATERIAL_NOZZLE_DIAMETER: &str = "material-nozzle-diameter";

    /// Flow rate of material per second (integer(1:MAX), OPTIONAL)
    pub const MATERIAL_RATE: &str = "material-rate";

    /// Units for material rate (type2 keyword, OPTIONAL)
    pub const MATERIAL_RATE_UNITS: &str = "material-rate-units";

    /// Whether filament retraction is used (boolean, OPTIONAL)
    pub const MATERIAL_RETRACTION: &str = "material-retraction";

    /// Exterior wall thickness in nanometers (integer(0:MAX), OPTIONAL)
    pub const MATERIAL_SHELL_THICKNESS: &str = "material-shell-thickness";

    /// Printing temperature (integer(-273:MAX), OPTIONAL)
    pub const MATERIAL_TEMPERATURE: &str = "material-temperature";
}

/// Strongly-typed representation of a printable 3D material (PWG 5100.21 §8.1.1)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Material {
    /// Human-readable, localized name (name(MAX), REQUIRED)
    pub name: String,

    /// Unlocalized key / identifier (keyword, REQUIRED)
    pub key: String,

    /// Material type (type2 keyword | name(MAX), REQUIRED)
    pub material_type: String,

    /// Fill density (integer(0:100), REQUIRED)
    pub fill_density: i32,

    /// Material purpose(s) (1setOf type2 keyword, REQUIRED)
    pub purpose: Vec<String>,

    /// Estimated or actual material amount (integer(0:MAX), OPTIONAL)
    pub amount: Option<i32>,

    /// Units for material amount (type2 keyword, OPTIONAL)
    pub amount_units: Option<String>,

    /// Material color (type2 keyword, OPTIONAL)
    pub color: Option<String>,

    /// Filament diameter in nanometers (integer(0:MAX), OPTIONAL)
    pub diameter: Option<i32>,

    /// Tolerance for filament diameter in nanometers (integer(0:MAX), OPTIONAL)
    pub diameter_tolerance: Option<i32>,

    /// Extruder nozzle diameter (integer(0:MAX), OPTIONAL)
    pub nozzle_diameter: Option<i32>,

    /// Flow rate of material per second (integer(1:MAX), OPTIONAL)
    pub rate: Option<i32>,

    /// Units for material rate (type2 keyword, OPTIONAL)
    pub rate_units: Option<String>,

    /// Whether filament retraction is used (boolean, OPTIONAL)
    pub retraction: Option<bool>,

    /// Exterior wall thickness in nanometers (integer(0:MAX), OPTIONAL)
    pub shell_thickness: Option<i32>,

    /// Printing temperature (integer(-273:MAX), OPTIONAL)
    pub temperature: Option<i32>,
}

impl Material {
    /// Converts this `Material` into an IPP `Collection` for use in `materials-col`.
    pub fn to_ipp(&self) -> IppValue {
        let mut map = BTreeMap::new();
        map.insert(tags::MATERIAL_NAME.into(), IppValue::TextWithoutLanguage(self.name.clone()));
        map.insert(tags::MATERIAL_KEY.into(), IppValue::TextWithoutLanguage(self.key.clone()));
        map.insert(tags::MATERIAL_TYPE.into(), IppValue::TextWithoutLanguage(self.material_type.clone()));
        map.insert(tags::MATERIAL_FILL_DENSITY.into(), IppValue::Integer(self.fill_density));
        map.insert(tags::MATERIAL_PURPOSE.into(), IppValue::Array(
            self.purpose.iter().map(|s| IppValue::TextWithoutLanguage(s.clone())).collect()
        ));
        if let Some(amount) = self.amount {
            map.insert(tags::MATERIAL_AMOUNT.into(), IppValue::Integer(amount));
        }
        if let Some(ref units) = self.amount_units {
            map.insert(tags::MATERIAL_AMOUNT_UNITS.into(), IppValue::TextWithoutLanguage(units.clone()));
        }
        if let Some(ref color) = self.color {
            map.insert(tags::MATERIAL_COLOR.into(), IppValue::TextWithoutLanguage(color.clone()));
        }
        if let Some(d) = self.diameter {
            map.insert(tags::MATERIAL_DIAMETER.into(), IppValue::Integer(d));
        }
        if let Some(dt) = self.diameter_tolerance {
            map.insert(tags::MATERIAL_DIAMETER_TOLERANCE.into(), IppValue::Integer(dt));
        }
        if let Some(nd) = self.nozzle_diameter {
            map.insert(tags::MATERIAL_NOZZLE_DIAMETER.into(), IppValue::Integer(nd));
        }
        if let Some(r) = self.rate {
            map.insert(tags::MATERIAL_RATE.into(), IppValue::Integer(r));
        }
        if let Some(ref ru) = self.rate_units {
            map.insert(tags::MATERIAL_RATE_UNITS.into(), IppValue::TextWithoutLanguage(ru.clone()));
        }
        if let Some(ret) = self.retraction {
            map.insert(tags::MATERIAL_RETRACTION.into(), IppValue::Boolean(ret));
        }
        if let Some(st) = self.shell_thickness {
            map.insert(tags::MATERIAL_SHELL_THICKNESS.into(), IppValue::Integer(st));
        }
        if let Some(temp) = self.temperature {
            map.insert(tags::MATERIAL_TEMPERATURE.into(), IppValue::Integer(temp));
        }
        IppValue::Collection(map)
    }

    /// Converts from an IPP attribute map into a `Material`.
    pub fn from_ipp(attrs: &AttrMap, group: AttributeGroup) -> Option<Self> {
        Some(Self {
            name: attrs.get_str(group, tags::MATERIAL_NAME)?.to_string(),
            key: attrs.get_str(group, tags::MATERIAL_KEY)?.to_string(),
            material_type: attrs.get_str(group, tags::MATERIAL_TYPE)?.to_string(),
            fill_density: attrs.get_i32(group, tags::MATERIAL_FILL_DENSITY)?,
            purpose: attrs.get_strings(group, tags::MATERIAL_PURPOSE).unwrap_or_default().into_iter().map(|v| v.to_string()).collect::<Vec<String>>(),
            amount: attrs.get_i32(group, tags::MATERIAL_AMOUNT),
            amount_units: attrs.get_str(group, tags::MATERIAL_AMOUNT_UNITS).map(str::to_owned),
            color: attrs.get_str(group, tags::MATERIAL_COLOR).map(str::to_owned),
            diameter: attrs.get_i32(group, tags::MATERIAL_DIAMETER),
            diameter_tolerance: attrs.get_i32(group, tags::MATERIAL_DIAMETER_TOLERANCE),
            nozzle_diameter: attrs.get_i32(group, tags::MATERIAL_NOZZLE_DIAMETER),
            rate: attrs.get_i32(group, tags::MATERIAL_RATE),
            rate_units: attrs.get_str(group, tags::MATERIAL_RATE_UNITS).map(str::to_owned),
            retraction: attrs.get_bool(group, tags::MATERIAL_RETRACTION),
            shell_thickness: attrs.get_i32(group, tags::MATERIAL_SHELL_THICKNESS),
            temperature: attrs.get_i32(group, tags::MATERIAL_TEMPERATURE),
        })
    }
}

/// represents a 3D printable object.
#[derive(Debug, Clone)]
pub struct PrintObject {
    /// document number (1:MAX)
    pub document: i32,
    /// the object offset on the 3D plane.
    pub offset: Vec3D,
    /// the size of the object
    pub size: Vec3D,
    /// object identifier (uri or uuid)
    pub object_id: String,
}

/// Describes the physical build volume of a 3D printer.
///
/// This defines the maximum printable dimensions and optional weight
/// constraints of the device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BuildVolume {
    /// Maximum printable width in millimeters (X axis).
    pub width: i32,

    /// Maximum printable depth in millimeters (Y axis).
    pub depth: i32,

    /// Maximum printable height in millimeters (Z axis).
    pub height: i32,

    /// Optional maximum supported weight in grams.
    pub max_weight: Option<i32>,
}

impl BuildVolume {
    /// Converts this `BuildVolume` into an IPP `Collection`.
    pub fn to_ipp(&self) -> IppValue {
        let mut map = BTreeMap::new();

        map.insert("width".into(), IppValue::Integer(self.width));
        map.insert("depth".into(), IppValue::Integer(self.depth));
        map.insert("height".into(), IppValue::Integer(self.height));

        if let Some(w) = self.max_weight {
            map.insert("max-weight".into(), IppValue::Integer(w));
        }

        IppValue::Collection(map)
    }
}

/// Represents a 3D print job ticket.
///
/// This struct encapsulates all settings required to submit a 3D print job
/// using IPP 3D Printing Extensions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobTicket {
    /// URI of the 3D model file to print.
    ///
    /// Typically a `3MF` file, but other formats may be supported.
    pub file_uri: String,

    /// Material to be used for the print.
    pub material: Material,

    /// Layer thickness in micrometers (µm).
    pub layer_resolution: i32,

    /// Infill density percentage (0–100).
    pub infill_density: i32,

    /// Whether support structures should be generated.
    pub supports: bool,
}

impl JobTicket {
    /// Converts this `JobTicket` into an IPP `Collection`.
    pub fn to_ipp(&self) -> IppValue {
        let mut map = BTreeMap::new();

        map.insert(
            "file-uri".into(),
            IppValue::TextWithoutLanguage(self.file_uri.clone()),
        );
        map.insert("material".into(), self.material.to_ipp());
        map.insert(
            "layer-resolution".into(),
            IppValue::Integer(self.layer_resolution),
        );
        map.insert(
            "infill-density".into(),
            IppValue::Integer(self.infill_density),
        );
        map.insert("supports".into(), IppValue::Boolean(self.supports));

        IppValue::Collection(map)
    }
}

/// Describes the capabilities of a 3D printer.
///
/// These attributes are typically returned by `Get-Printer-Attributes` and
/// describe what materials and print parameters the device supports.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrinterCapabilities {
    /// List of materials supported by the printer.
    pub supported_materials: Vec<Material>,

    /// Physical build volume constraints of the printer.
    pub build_volume: BuildVolume,

    /// Minimum supported layer resolution in micrometers (µm).
    pub min_layer_resolution: i32,

    /// Maximum supported layer resolution in micrometers (µm).
    pub max_layer_resolution: i32,

    /// Maximum print speed in millimeters per second (mm/s).
    pub max_print_speed: i32,
    /// what is the printers optimal humidity as a percentage (0:100)
    pub chamber_humidity: Option<i32>,

    /// what is the optimal temperature for this printer's chamber range (-273:MAX) in degrees Celcius.
    pub chamber_temperature: Option<i32>,
    /// what is the optimal chamber temperature set this printer supports an array of either Integer or Range.
    pub chamber_temperature_supported: Option<IntegerRange>,
}

impl PrinterCapabilities {
    /// Converts this `PrinterCapabilities` into an IPP `Collection`.
    pub fn to_ipp(&self) -> IppValue {
        let mut map = BTreeMap::new();

        map.insert(
            "supported-materials".into(),
            IppValue::Array(
                self.supported_materials
                    .iter()
                    .map(|m| m.to_ipp())
                    .collect(),
            ),
        );

        map.insert("build-volume".into(), self.build_volume.to_ipp());
        map.insert(
            "min-layer-resolution".into(),
            IppValue::Integer(self.min_layer_resolution),
        );
        map.insert(
            "max-layer-resolution".into(),
            IppValue::Integer(self.max_layer_resolution),
        );
        map.insert(
            "max-print-speed".into(),
            IppValue::Integer(self.max_print_speed),
        );

        IppValue::Collection(map)
    }
}
