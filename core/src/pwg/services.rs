//! System Service IPP attribute types and helpers.
//!
//! This module defines strongly-typed representations of IPP System Service
//! attributes (printers, jobs, and resources), with first-class support for
//! multilingual human-readable text as defined in RFC 8011.
//!
//! ## Design goals
//!
//! * **Correct IPP typing**
//!   * Human-readable UI strings use `text` / `textWithLanguage`
//!   * Identifiers, enums, and protocol values use `textWithoutLanguage`
//! * **Centralized localization**
//!   * All localized text is encoded via `make_ipp_text`
//! * **System-service friendly**
//!   * Matches PWG System Service concepts (RFC 3995/3996 family)
//!
//! This module is intentionally conservative: it models only *system-level*
//! views and does not attempt to replace printer- or job-specific IPP schemas.

use std::collections::BTreeMap;

use crate::ipputils::{AttrMap, AttributeGroup, LangTag, make_ipp_text};
use crate::rfc::reasons::PrinterStateReason;
use ipp::prelude::IppValue;
use std::str::FromStr;

/// Represents the operational state of a system-level printer.
///
/// This reflects the high-level runtime status of the printer and is typically
/// exposed via system service attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemPrinterState {
    /// The printer is idle and ready to accept jobs.
    Idle,

    /// The printer is actively processing one or more jobs.
    Processing,

    /// The printer is stopped due to an error or operator intervention.
    Stopped,

    /// The printer is undergoing maintenance or servicing.
    Maintenance,
}

/// Represents the operational state of a system-level job.
///
/// Job states are reported in job and system service queries and may be
/// accompanied by one or more explanatory reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemJobState {
    /// The job has been accepted but has not started processing.
    Pending,

    /// The job is currently being processed.
    Processing,

    /// The job completed successfully.
    Completed,

    /// The job was canceled by a user or administrator.
    Canceled,

    /// The job was aborted due to an error or system failure.
    Aborted,
}

impl std::str::FromStr for SystemPrinterState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "idle" => Ok(Self::Idle),
            "processing" => Ok(Self::Processing),
            "stopped" => Ok(Self::Stopped),
            "maintenance" => Ok(Self::Maintenance),
            _ => Err(()),
        }
    }
}

impl std::str::FromStr for SystemJobState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "canceled" => Ok(Self::Canceled),
            "aborted" => Ok(Self::Aborted),
            _ => Err(()),
        }
    }
}

/// System-level printer attributes as defined by the IPP System Service.
///
/// These attributes describe the identity, status, and configuration of a
/// printer system and are typically returned by `Get-System-Attributes`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPrinterAttributes {
    /// Human-readable make and model of the printer system.
    ///
    /// Example: `"Prusa MK4"` or `"HP Jet Fusion 5200"`.
    pub system_make_and_model: String,

    /// User-facing name of the printer system.
    ///
    /// This value may be localized and is intended for display in UIs.
    pub system_name: String,

    /// Optional description of the printer's physical or logical location.
    ///
    /// Example: `"Lab A, Room 203"`.
    pub system_location: Option<String>,

    /// Optional contact information for the system administrator.
    ///
    /// Example: `"mailto:admin@example.org"` or `"Ops Team ext. 42"`.
    pub system_contact: Option<String>,

    /// Time in seconds since the system was last booted.
    ///
    /// This value is relative and MUST NOT be interpreted as wall-clock time.
    pub system_up_time: Option<u64>,

    /// Current operational state of the printer system.
    pub system_state: SystemPrinterState,

    /// Human-readable reasons explaining the current system state.
    ///
    /// These strings are intended for operator diagnostics and UI display.
    pub system_state_reasons: Vec<PrinterStateReason>,

    /// Network addresses associated with the printer system.
    ///
    /// May include IP literals, hostnames, or transport-specific addresses.
    pub system_network_addresses: Vec<String>,

    /// Optional software or firmware version running on the system.
    pub system_software_version: Option<String>,

    /// Optional manufacturer-assigned serial number.
    pub system_serial_number: Option<String>,

    /// Optional language used for all human-readable text attributes.
    ///
    /// When present, this language tag is applied to all
    /// `TextWithLanguage` attributes produced by `to_ipp()`.
    pub language: Option<LangTag>,
}

impl SystemPrinterAttributes {
    /// Converts this struct into an IPP `Collection` value.
    ///
    /// Human-readable fields are encoded using `TextWithLanguage` when a
    /// language is present, otherwise `TextWithoutLanguage`.
    pub fn to_ipp(&self) -> IppValue {
        let mut map = BTreeMap::new();
        let lang = &self.language;

        map.insert(
            "system-make-and-model".into(),
            make_ipp_text(lang, &self.system_make_and_model),
        );

        map.insert("system-name".into(), make_ipp_text(lang, &self.system_name));

        if let Some(loc) = &self.system_location {
            map.insert("system-location".into(), make_ipp_text(lang, loc));
        }

        if let Some(contact) = &self.system_contact {
            map.insert("system-contact".into(), make_ipp_text(lang, contact));
        }

        if let Some(up) = self.system_up_time {
            map.insert("system-up-time".into(), IppValue::Integer(up as i32));
        }

        map.insert(
            "system-state".into(),
            IppValue::TextWithoutLanguage(format!("{:?}", self.system_state)),
        );

        map.insert(
            "system-state-reasons".into(),
            IppValue::Array(
                self.system_state_reasons
                    .iter()
                    .map(|r| make_ipp_text(lang, &r.to_string()))
                    .collect(),
            ),
        );

        if let Some(ver) = &self.system_software_version {
            map.insert(
                "system-software-version".into(),
                IppValue::TextWithoutLanguage(ver.clone()),
            );
        }

        if let Some(sn) = &self.system_serial_number {
            map.insert(
                "system-serial-number".into(),
                IppValue::TextWithoutLanguage(sn.clone()),
            );
        }

        map.insert(
            "system-network-addresses".into(),
            IppValue::Array(
                self.system_network_addresses
                    .iter()
                    .map(|a| IppValue::TextWithoutLanguage(a.clone()))
                    .collect(),
            ),
        );

        IppValue::Collection(map)
    }

    /// convert the underlying type from ipp attribute map.
    pub fn from_ipp(attrs: &AttrMap) -> Option<Self> {
        let group = AttributeGroup::SystemAttrs;
        let language: Option<LangTag> = attrs
            .get_str(group, "attributes-natural-language")
            .and_then(|v| v.parse().ok());

        Some(Self {
            system_make_and_model: attrs
                .get_str(group, "system-make-and-model")
                .map(|v| v.to_string())?,
            system_name: attrs.get_str(group, "system-name").map(|v| v.to_string())?,
            system_location: attrs
                .get_str(group, "system-location")
                .map(|v| v.to_string()),
            system_contact: attrs
                .get_str(group, "system-contact")
                .map(|v| v.to_string()),
            system_up_time: attrs.get_i32(group, "system-up-time").map(|v| v as u64),
            system_state: attrs.get_str(group, "system-state")?.parse().ok()?,
            system_state_reasons: attrs
                .get_strings(group, "system-state-reasons")
                .unwrap_or_default()
                .into_iter()
                .map(|v| PrinterStateReason::from_str(v).expect("failed on infallible"))
                .collect(),
            system_network_addresses: attrs
                .get_strings(group, "system-network-addresses")
                .unwrap_or_default()
                .into_iter()
                .map(str::to_owned)
                .collect(),
            system_software_version: attrs
                .get_str(group, "system-software-version")
                .map(str::to_owned),
            system_serial_number: attrs
                .get_str(group, "system-serial-number")
                .map(str::to_owned),
            language,
        })
    }
}

/// System-level job attributes.
///
/// These attributes describe the lifecycle, ownership, and accounting
/// properties of a job as seen from the system service perspective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemJobAttributes {
    /// System-unique identifier for the job.
    pub job_id: String,

    /// Current operational state of the job.
    pub job_state: SystemJobState,

    /// Human-readable reasons explaining the job's current state.
    pub job_state_reasons: Vec<String>,

    /// Optional name of the user that originated the job.
    pub job_originating_user: Option<String>,

    /// Size of the job data in kilobytes (k-octets).
    ///
    /// This value is typically reported for accounting purposes.
    pub job_k_octets: Option<i32>,

    /// Job creation time expressed as seconds since the Unix epoch.
    pub job_creation_time: Option<u64>,

    /// Job completion time expressed as seconds since the Unix epoch.
    pub job_completion_time: Option<u64>,

    /// Optional language used for human-readable job attributes.
    pub language: Option<LangTag>,
}

impl SystemJobAttributes {
    /// Converts this struct into an IPP `Collection` value.
    pub fn to_ipp(&self) -> IppValue {
        let mut map = BTreeMap::new();
        let lang = &self.language;

        map.insert(
            "job-id".into(),
            IppValue::TextWithoutLanguage(self.job_id.clone()),
        );

        map.insert(
            "job-state".into(),
            IppValue::TextWithoutLanguage(format!("{:?}", self.job_state)),
        );

        map.insert(
            "job-state-reasons".into(),
            IppValue::Array(
                self.job_state_reasons
                    .iter()
                    .map(|r| make_ipp_text(lang, r))
                    .collect(),
            ),
        );

        if let Some(user) = &self.job_originating_user {
            map.insert(
                "job-originating-user-name".into(),
                make_ipp_text(lang, user),
            );
        }

        if let Some(k) = self.job_k_octets {
            map.insert("job-k-octets".into(), IppValue::Integer(k));
        }

        if let Some(t) = self.job_creation_time {
            map.insert("job-creation-time".into(), IppValue::Integer(t as i32));
        }

        if let Some(t) = self.job_completion_time {
            map.insert("job-completion-time".into(), IppValue::Integer(t as i32));
        }

        IppValue::Collection(map)
    }

    /// Constructs `SystemJobAttributes` from an IPP attribute map.
    ///
    /// Returns `None` if required attributes are missing or invalid.
    pub fn from_ipp(attrs: &AttrMap) -> Option<Self> {
        let group = AttributeGroup::SystemAttrs;

        let language: Option<LangTag> = attrs
            .get_str(group, "attributes-natural-language")
            .and_then(|v| v.parse().ok());

        Some(Self {
            job_id: attrs.get_str(group, "job-id").map(str::to_owned)?,

            job_state: attrs.get_str(group, "job-state")?.parse().ok()?,

            job_state_reasons: attrs
                .get_strings(group, "job-state-reasons")
                .unwrap_or_default()
                .into_iter()
                .map(str::to_owned)
                .collect(),

            job_originating_user: attrs
                .get_str(group, "job-originating-user-name")
                .map(str::to_owned),

            job_k_octets: attrs.get_i32(group, "job-k-octets"),

            job_creation_time: attrs.get_i32(group, "job-creation-time").map(|v| v as u64),

            job_completion_time: attrs
                .get_i32(group, "job-completion-time")
                .map(|v| v as u64),

            language,
        })
    }
}

/// System-managed physical or consumable resource.
///
/// Examples include material reservoirs, trays, toolheads, or build platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceAttributes {
    /// Unique identifier for the resource within the system.
    pub resource_id: String,

    /// Resource type identifier.
    ///
    /// Example values: `"material"`, `"tray"`, `"toolhead"`.
    pub resource_type: String,

    /// Human-readable description of the resource.
    pub resource_description: Option<String>,

    /// Current resource level expressed as a percentage or unit count.
    pub resource_current_level: Option<i32>,

    /// Maximum capacity of the resource expressed in the same units
    /// as `resource_current_level`.
    pub resource_max_capacity: Option<i32>,

    /// Optional language used for human-readable resource attributes.
    pub language: Option<LangTag>,
}

impl SystemResourceAttributes {
    /// Converts this struct into an IPP `Collection` value.
    pub fn to_ipp(&self) -> IppValue {
        let mut map = BTreeMap::new();
        let lang = &self.language;

        map.insert(
            "resource-id".into(),
            IppValue::TextWithoutLanguage(self.resource_id.clone()),
        );

        map.insert(
            "resource-type".into(),
            IppValue::TextWithoutLanguage(self.resource_type.clone()),
        );

        if let Some(desc) = &self.resource_description {
            map.insert("resource-description".into(), make_ipp_text(lang, desc));
        }

        if let Some(level) = self.resource_current_level {
            map.insert("resource-current-level".into(), IppValue::Integer(level));
        }

        if let Some(max) = self.resource_max_capacity {
            map.insert("resource-max-capacity".into(), IppValue::Integer(max));
        }

        IppValue::Collection(map)
    }

    /// Constructs `SystemResourceAttributes` from an IPP attribute map.
    ///
    /// Returns `None` if required attributes are missing.
    pub fn from_ipp(attrs: &AttrMap) -> Option<Self> {
        let group = AttributeGroup::SystemAttrs;

        let language: Option<LangTag> = attrs
            .get_str(group, "attributes-natural-language")
            .and_then(|v| v.parse().ok());

        Some(Self {
            resource_id: attrs.get_str(group, "resource-id").map(str::to_owned)?,

            resource_type: attrs.get_str(group, "resource-type").map(str::to_owned)?,

            resource_description: attrs
                .get_str(group, "resource-description")
                .map(str::to_owned),

            resource_current_level: attrs.get_i32(group, "resource-current-level"),

            resource_max_capacity: attrs.get_i32(group, "resource-max-capacity"),

            language,
        })
    }
}