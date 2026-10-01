//! RFC 3995 — IPP Job Accounting
//!
//! This module defines a typed representation of IPP job accounting
//! attributes as specified in RFC 3995. It is transport-agnostic and
//! suitable for persistence in databases or auditing systems.
//!
//! please see [RFC 3995](https://www.rfc-editor.org/rfc/rfc3995) for more information.

use crate::ipputils::AttrMap;
use crate::ipputils::AttributeGroup;

/// Typed job accounting information derived from IPP attributes.
///
/// All fields are optional because devices MAY report only a subset
/// of accounting attributes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct JobAccounting {
    /// Total number of impressions (logical pages).
    pub impressions: Option<i32>,

    /// Total number of physical media sheets used.
    pub media_sheets: Option<i32>,

    /// Size of job data in octets.
    pub job_k_octets: Option<i32>,

    /// Processing time in seconds.
    pub time_seconds: Option<i32>,

    /// Device-reported accounting extensions.
    ///
    /// This preserves forward compatibility and vendor-specific metrics.
    pub extensions: Vec<AccountingExtension>,
}

/// A single IPP accounting extension attribute that is not explicitly modeled.
///
/// This struct represents an **opaque accounting attribute** used in IPP job or
/// printer accounting contexts when a first-class Rust type is not provided.
/// It allows implementers to preserve and forward vendor-specific or
/// future-standard accounting fields without losing information.
///
/// ## Context
///
/// IPP accounting attributes are used to report **resource usage, billing data,
/// and operational metrics** associated with a print job or printer. While many
/// common attributes are standardized, IPP explicitly allows extensions via
/// additional attribute names.
///
/// This type exists to support:
///
/// - Vendor-defined accounting attributes
/// - Experimental or draft PWG extensions
/// - Forward compatibility with future RFCs / PWG specs
///
/// ## Semantics
///
/// - `name` is the full IPP attribute name (e.g. `"job-k-octets"`,
///   `"job-impressions"`, `"vendor-energy-used"`).
/// - `value` is the numeric value of the attribute, interpreted as a signed
///   64-bit integer to accommodate large counters and cumulative metrics.
///
/// The meaning and unit of `value` are **defined by the attribute name** and
/// relevant specification or vendor documentation.
///
/// ## Mapping to IPP
///
/// When serialized into IPP, this struct is typically encoded as:
///
/// ```text
/// <name> = integer(<value>)
/// ```
///
/// and included in a job or printer accounting attribute group.
///
/// ## Example
///
/// ```rust
/// let ext = AccountingExtension {
///     name: "job-k-octets".to_string(),
///     value: 2048,
/// };
/// ```
///
/// ## Design Notes
///
/// - This struct intentionally avoids strong typing to prevent premature
///   standardization of accounting semantics.
/// - Consumers may choose to post-process known attribute names into
///   strongly-typed representations.
/// - Unknown attributes SHOULD be preserved when proxying or aggregating IPP
///   accounting data.
///
/// ## References
///
/// - RFC 8010 – IPP/1.1 Model and Semantics  
/// - RFC 8011 – IPP/1.1 Encoding and Transport  
/// - RFC 3996 – IPP Event Notifications (accounting-related events)
#[derive(Debug, Clone, PartialEq)]
#[warn(missing_docs)]
pub struct AccountingExtension {
    /// The IPP attribute name (e.g., `"job-k-octets"`).
    pub name: String,

    /// The integer value associated with the accounting attribute.
    ///
    /// The unit and interpretation are defined by the attribute name and
    /// applicable specification.
    pub value: i64,
}

impl JobAccounting {
    /// Construct a `JobAccounting` instance from IPP attributes.
    ///
    /// This reads from the Job Attributes group as defined by RFC 3995.
    pub fn from_attrs(attrs: &AttrMap) -> Self {
        let group = AttributeGroup::JobAttrs;

        let impressions = attrs.get_i32(group, "job-impressions-completed");

        let media_sheets = attrs.get_i32(group, "job-media-sheets-completed");

        let job_k_octets = attrs.get_i32(group, "job-k-octets").map(|v| v * 1024);

        let time_seconds = attrs.get_i32(group, "job-time");

        JobAccounting {
            impressions,
            media_sheets,
            job_k_octets,
            time_seconds,
            extensions: Vec::new(),
        }
    }
}
