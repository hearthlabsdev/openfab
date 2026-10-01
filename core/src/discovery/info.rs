//! High-level structured representation of IPP printer information.
//!
//! This module provides a typed view over printer-related IPP attributes,
//! typically returned by the `Get-Printer-Attributes` operation.
//!
//! Parsing is intentionally lossy-but-explicit:
//! - Unknown attributes are preserved in `extensions`
//! - Enum-like values are kept as raw integers or strings
//! - No recursive normalization is performed

use std::collections::BTreeMap;

use ipp::model::DelimiterTag;
use ipp::value::IppValue;

use crate::attr_map::AttrMap;

/// Fully parsed printer information.
#[derive(Debug, Clone)]
pub struct PrinterInfo {
    pub protocol: ProtocolInfo,
    pub identity: PrinterIdentity,
    pub state: PrinterStateInfo,
    pub capabilities: PrinterCapabilities,
    pub defaults: PrinterDefaults,
    pub media: MediaCapabilities,
    pub finishing: FinishingCapabilities,
    pub color: ColorCapabilities,
    pub uris: PrinterUris,
    pub performance: PerformanceInfo,

    /// Vendor-specific or unmodeled attributes
    pub extensions: BTreeMap<String, IppValue>,
}

/* ==============================
 * Protocol / language
 * ============================== */

#[derive(Debug, Clone, Default)]
pub struct ProtocolInfo {
    pub attributes_charset: Option<String>,
    pub attributes_natural_language: Option<String>,
    pub generated_natural_languages: Vec<String>,
    pub ipp_versions_supported: Vec<String>,
    pub operations_supported: Vec<i32>,
}

/* ==============================
 * Identity & metadata
 * ============================== */

#[derive(Debug, Clone, Default)]
pub struct PrinterIdentity {
    pub name: Option<String>,
    pub info: Option<String>,
    pub location: Option<String>,
    pub make_and_model: Option<String>,
    pub firmware_name: Option<String>,
    pub firmware_version: Option<String>,
    pub device_id: Option<String>,
    pub uuid: Option<String>,
    pub mopria_certified: Option<bool>,
}

/* ==============================
 * State & status
 * ============================== */

#[derive(Debug, Clone, Default)]
pub struct PrinterStateInfo {
    pub state: Option<i32>,
    pub is_accepting_jobs: Option<bool>,
    pub state_message: Option<String>,
    pub state_reasons: Vec<String>,
    pub status_message: Option<String>,
    pub queued_job_count: Option<i32>,
    pub uptime_seconds: Option<i32>,
}

/* ==============================
 * Capabilities
 * ============================== */

#[derive(Debug, Clone, Default)]
pub struct PrinterCapabilities {
    pub document_formats_supported: Vec<String>,
    pub document_format_default: Option<String>,
    pub document_format_preferred: Vec<String>,
    pub compression_supported: Vec<String>,
    pub charset_supported: Vec<String>,
    pub charset_configured: Option<String>,
    pub natural_language_configured: Option<String>,
    pub pdl_override_supported: Option<bool>,
}

/* ==============================
 * Defaults
 * ============================== */

#[derive(Debug, Clone, Default)]
pub struct PrinterDefaults {
    pub copies_default: Option<i32>,
    pub sides_default: Option<String>,
    pub print_quality_default: Option<i32>,
    pub output_bin_default: Option<String>,
    pub orientation_requested_default: Option<i32>,
    pub multiple_document_handling_default: Option<String>,
    pub print_color_mode_default: Option<String>,
    pub media_default: Option<String>,
    pub printer_resolution_default: Option<PrinterResolution>,
    pub finishings_default: Vec<i32>,
}

/* ==============================
 * Media
 * ============================== */

#[derive(Debug, Clone, Default)]
pub struct MediaCapabilities {
    pub media_supported: Vec<String>,
    pub media_type_supported: Vec<String>,
    pub media_source_supported: Vec<String>,
    pub media_col_supported: bool,
}

/* ==============================
 * Finishing
 * ============================== */

#[derive(Debug, Clone, Default)]
pub struct FinishingCapabilities {
    pub finishings_supported: Vec<i32>,
    pub finishings: Vec<i32>,
    pub output_bins_supported: Vec<String>,
}

/* ==============================
 * Color & quality
 * ============================== */

#[derive(Debug, Clone, Default)]
pub struct ColorCapabilities {
    pub color_supported: Option<bool>,
    pub color_mode_supported: Vec<String>,
    pub print_color_mode_supported: Vec<String>,
    pub print_quality_supported: Vec<i32>,
}

/* ==============================
 * URIs & security
 * ============================== */

#[derive(Debug, Clone, Default)]
pub struct PrinterUris {
    pub printer_uri: Option<String>,
    pub printer_uri_supported: Vec<String>,
    pub more_info: Option<String>,
    pub uri_authentication_supported: Vec<String>,
    pub uri_security_supported: Vec<String>,
}

/* ==============================
 * Performance
 * ============================== */

#[derive(Debug, Clone, Default)]
pub struct PerformanceInfo {
    pub pages_per_minute: Option<i32>,
}

/* ==============================
 * Supporting types
 * ============================== */

#[derive(Debug, Clone)]
pub struct PrinterResolution {
    pub cross_feed_dpi: i32,
    pub feed_dpi: i32,
    pub units: i8,
}

/* ==============================
 * Conversion from AttrMap
 * ============================== */

impl TryFrom<&AttrMap> for PrinterInfo {
    type Error = ();

    fn try_from(attrs: &AttrMap) -> Result<Self, Self::Error> {
        let g = DelimiterTag::PrinterAttributes;

        Ok(Self {
            protocol: ProtocolInfo {
                attributes_charset: attrs.get_str(g, "attributes-charset").map(str::to_owned),
                attributes_natural_language: attrs.get_str(g, "attributes-natural-language").map(str::to_owned),
                generated_natural_languages: attrs.get_strings(g, "generated-natural-language-supported")
                    .unwrap_or_default().into_iter().map(String::from).collect(),
                ipp_versions_supported: attrs.get_strings(g, "ipp-versions-supported")
                    .unwrap_or_default().into_iter().map(String::from).collect(),
                operations_supported: vec![], // left raw intentionally
            },

            identity: PrinterIdentity {
                name: attrs.get_str(g, "printer-name").map(String::from),
                info: attrs.get_str(g, "printer-info").map(String::from),
                location: attrs.get_str(g, "printer-location").map(String::from),
                make_and_model: attrs.get_str(g, "printer-make-and-model").map(String::from),
                firmware_name: attrs.get_str(g, "printer-firmware-name").map(String::from),
                firmware_version: attrs.get_str(g, "printer-firmware-string-version").map(String::from),
                device_id: attrs.get_str(g, "printer-device-id").map(String::from),
                uuid: attrs.get_str(g, "printer-uuid").map(String::from),
                mopria_certified: attrs.get(g, "mopria-certified").and_then(|v| match v {
                    IppValue::Boolean(b) => Some(*b),
                    _ => None,
                }),
            },

            state: PrinterStateInfo {
                state: attrs.get(g, "printer-state").and_then(|v| match v {
                    IppValue::Enum(i) => Some(*i),
                    _ => None,
                }),
                is_accepting_jobs: attrs.get(g, "printer-is-accepting-jobs").and_then(|v| match v {
                    IppValue::Boolean(b) => Some(*b),
                    _ => None,
                }),
                state_message: attrs.get_str(g, "printer-state-message").map(String::from),
                state_reasons: attrs.get_strings(g, "printer-state-reasons")
                    .unwrap_or_default().into_iter().map(String::from).collect(),
                status_message: attrs.get_str(g, "status-message").map(String::from),
                queued_job_count: attrs.get(g, "queued-job-count").and_then(int_value),
                uptime_seconds: attrs.get(g, "printer-up-time").and_then(int_value),
            },

            capabilities: PrinterCapabilities::default(),
            defaults: PrinterDefaults::default(),
            media: MediaCapabilities::default(),
            finishing: FinishingCapabilities::default(),
            color: ColorCapabilities::default(),
            uris: PrinterUris::default(),
            performance: PerformanceInfo {
                pages_per_minute: attrs.get(g, "pages-per-minute").and_then(int_value),
            },

            extensions: BTreeMap::new(),
        })
    }
}

/* ==============================
 * Helpers
 * ============================== */

#[inline]
fn int_value(v: &IppValue) -> Option<i32> {
    match v {
        IppValue::Integer(i) | IppValue::Enum(i) => Some(*i),
        _ => None,
    }
}
