//! IPP attribute parsing and lookup utilities.
//!
//! This module provides a flattened, ergonomic view over [`ipp::attribute::IppAttributes`]
//! suitable for convenient attribute lookup by *(group, attribute-name)* pairs.
//!
//! ## Design overview
//!
//! IPP attributes are natively organized into *attribute groups* (identified by
//! [`AttributeGroup`]), each containing zero or more named attributes. While this
//! structure is faithful to the protocol, it can be cumbersome to work with
//! directly when performing repeated lookups.
//!
//! [`AttrMap`] flattens the grouped structure into a single `HashMap` keyed by
//! [`GroupKey`] `(group, name)` pairs while preserving the original
//! [`IppAttribute`] values.
//!
//! This design favors:
//! - **Correctness**: group boundaries are preserved via [`AttributeGroup`]
//! - **Clarity**: lookups are explicit about which group they target
//! - **Safety**: no borrowed-key tricks or `unsafe` code
//! - **Predictability**: no implicit type coercions or recursive flattening
//!
//! ## String handling
//!
//! IPP defines many string-like value types. This module provides helpers
//! that normalize these into `&str` where appropriate:
//!
//! - `get_str` extracts a single string value
//! - `get_strings` extracts a list of strings from arrays or collections
//!
//! These helpers are intentionally **shallow**:
//! - Arrays are handled at one level
//! - Collections are treated as flat key/value containers
//! - Nested arrays are *not* recursively flattened
//!
//! This matches typical, spec-compliant IPP usage and avoids surprising behavior.

use chrono::{DateTime, Utc};
use ipp::attribute::{IppAttribute, IppAttributes};
use ipp::model::DelimiterTag;
use ipp::value::IppValue;
use language_tags::LanguageTag;
use serde::{Deserialize, Serialize, Serializer, de::Deserializer};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

/// A thin, strongly-typed wrapper around `LanguageTag`.
///
/// This type exists to provide:
///
/// - Ergonomic trait implementations (`AsRef`, `Borrow`, `Deref`)
/// - Stable Serde serialization via the canonical BCP 47 string form
/// - A distinct type for APIs that require language-tag semantics
///
/// Internally, this wraps `language_tags::LanguageTag` and delegates
/// parsing and formatting to its `FromStr` and `Display` implementations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LangTag {
    tag: LanguageTag,
}

/// Helper to construct an IPP text value with optional natural language.
///
/// * If `lang` is `Some`, produces `TextWithLanguage`
/// * Otherwise, produces `TextWithoutLanguage`
pub fn make_ipp_text(lang: &Option<LangTag>, text: &str) -> IppValue {
    match lang {
        Some(lang) => IppValue::TextWithLanguage {
            language: lang.to_string(),
            text: text.to_string(),
        },
        None => IppValue::TextWithoutLanguage(text.to_string()),
    }
}

impl LangTag {
    /// Creates a new `LangTag` from a `LanguageTag`.
    pub fn new(tag: LanguageTag) -> Self {
        Self { tag }
    }

    /// Returns a reference to the inner `LanguageTag`.
    pub fn inner(&self) -> &LanguageTag {
        &self.tag
    }
}

/* ---------- Standard trait forwarding ---------- */

impl Deref for LangTag {
    type Target = LanguageTag;

    fn deref(&self) -> &Self::Target {
        &self.tag
    }
}

impl AsRef<LanguageTag> for LangTag {
    fn as_ref(&self) -> &LanguageTag {
        &self.tag
    }
}

impl Borrow<LanguageTag> for LangTag {
    fn borrow(&self) -> &LanguageTag {
        &self.tag
    }
}

/* ---------- Display / FromStr ---------- */

impl fmt::Display for LangTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tag.fmt(f)
    }
}

impl FromStr for LangTag {
    type Err = <LanguageTag as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LanguageTag::from_str(s).map(Self::new)
    }
}

/* ---------- Serde integration ---------- */

impl Serialize for LangTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for LangTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        LangTag::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// A string with an associated natural language (RFC 8011 §4.1.8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedString {
    /// the text value
    pub value: String,
    /// an optional language specification e.g. "en", "fr"
    pub language: Option<LangTag>, // e.g., "en", "fr", "es"
}

impl LocalizedString {
    /// convert the localized string into an IppValue matching based on whether or not a language code exists in self
    pub fn to_ipp(&self) -> IppValue {
        // In IPP, localized strings are sent as `textWithLanguage` value
        match &self.language {
            Some(lang) => IppValue::TextWithLanguage {
                text: self.value.clone(),
                language: lang.to_string(),
            },
            None => IppValue::TextWithoutLanguage(self.value.clone()), // fallback
        }
    }
}

/// IPP Attribute Group Tags
///
/// Values are defined by IANA:
/// <https://www.iana.org/assignments/ipp-registrations/>
///
/// These tags delimit attribute groups in IPP binary encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttributeGroupTag(pub u8);

impl AttributeGroupTag {
    /// Reserved (not used)
    pub const RESERVED: Self = Self(0x00);

    /// Operation Attributes
    /// RFC 8010 §3.1
    pub const OPERATION_ATTRIBUTES: Self = Self(0x01);

    /// Job Attributes
    /// RFC 8010 §3.1
    pub const JOB_ATTRIBUTES: Self = Self(0x02);

    /// End of Attributes
    /// RFC 8010 §3.1
    pub const END_OF_ATTRIBUTES: Self = Self(0x03);

    /// Printer Attributes
    /// RFC 8010 §3.1
    pub const PRINTER_ATTRIBUTES: Self = Self(0x04);

    /// Unsupported Attributes
    /// RFC 8010 §3.1
    pub const UNSUPPORTED: Self = Self(0x05);

    /// Subscription Attributes
    /// RFC 3995
    pub const SUBSCRIPTION_ATTRIBUTES: Self = Self(0x06);

    /// Event Notification Attributes
    /// RFC 3995
    pub const EVENT_NOTIFICATION_ATTRIBUTES: Self = Self(0x07);

    /// Resource Attributes
    /// PWG 5100.22 (System Service)
    pub const RESOURCE_ATTRIBUTES: Self = Self(0x08);

    /// Document Attributes
    /// PWG 5100.5 (IPP Document Object)
    pub const DOCUMENT_ATTRIBUTES: Self = Self(0x09);

    /// System Attributes
    /// PWG 5100.22 (System Service)
    pub const SYSTEM_ATTRIBUTES: Self = Self(0x0A);

    /// Vendor / Future Extensions (0x0B–0x0F)
    pub const fn vendor(value: u8) -> Self {
        Self(value)
    }

    /// displays the name of the attribute tag
    pub fn name(self) -> &'static str {
        match self.0 {
            0x00 => "reserved",
            0x01 => "operation-attributes-tag",
            0x02 => "job-attributes-tag",
            0x03 => "end-of-attributes-tag",
            0x04 => "printer-attributes-tag",
            0x05 => "unsupported-attributes-tag",
            0x06 => "subscription-attributes-tag",
            0x07 => "event-notification-attributes-tag",
            0x08 => "resource-attributes-tag",
            0x09 => "document-attributes-tag",
            0x0A => "system-attributes-tag",
            _ => "vendor-extension",
        }
    }
}

impl From<u8> for AttributeGroupTag {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<AttributeGroupTag> for u8 {
    #[inline]
    fn from(tag: AttributeGroupTag) -> Self {
        tag.0
    }
}

/// IPP Attribute Group Tag
/// Covers all known tags from IANA / RFCs / PWG specs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttributeGroup {
    /// Reserved
    Reserved,
    /// Operation Attributes (RFC 8010 / 2910)
    OpAttrs,
    /// Job Attributes (RFC 8010 / 2910)
    JobAttrs,
    /// End of Attributes (RFC 8010 / 2910)
    EndOfAttributes,
    /// Printer Attributes (RFC 8010 / 2910)
    PrinterAttrs,
    /// Unsupported Attributes (RFC 8010 / 2910)
    Unsupported,
    /// Subscription Attributes (RFC 3995)
    SubscriptionAttrs,
    /// Event Notification Attributes (RFC 3995)
    EventNotificationAttrs,
    /// Resource Attributes (PWG 5100.22 System Service)
    ResourceAttrs,
    /// Document Attributes (PWG 5100.5 Document Object)
    DocumentAttrs,
    /// System Attributes (PWG 5100.22 System Service)
    SystemAttrs,
    // future extensions 0x0B–0x0F can be handled as `Unknown(u8)`
}

/// represents a container around a u8 for handling unknown attribute groups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnknownGroup(u8);

impl TryFrom<u8> for AttributeGroup {
    type Error = UnknownGroup;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let tag = AttributeGroupTag(value);
        match tag {
            AttributeGroupTag(0x00) => Ok(AttributeGroup::Reserved),
            AttributeGroupTag::OPERATION_ATTRIBUTES => Ok(AttributeGroup::OpAttrs),
            AttributeGroupTag::JOB_ATTRIBUTES => Ok(AttributeGroup::JobAttrs),
            AttributeGroupTag::END_OF_ATTRIBUTES => Ok(AttributeGroup::EndOfAttributes),
            AttributeGroupTag::PRINTER_ATTRIBUTES => Ok(AttributeGroup::PrinterAttrs),
            AttributeGroupTag::UNSUPPORTED => Ok(AttributeGroup::Unsupported),
            AttributeGroupTag::SUBSCRIPTION_ATTRIBUTES => Ok(AttributeGroup::SubscriptionAttrs),
            AttributeGroupTag::EVENT_NOTIFICATION_ATTRIBUTES => {
                Ok(AttributeGroup::EventNotificationAttrs)
            }
            AttributeGroupTag::RESOURCE_ATTRIBUTES => Ok(AttributeGroup::ResourceAttrs),
            AttributeGroupTag::DOCUMENT_ATTRIBUTES => Ok(AttributeGroup::DocumentAttrs),
            AttributeGroupTag::SYSTEM_ATTRIBUTES => Ok(AttributeGroup::SystemAttrs),
            _ => Err(UnknownGroup(value)),
        }
    }
}

impl From<AttributeGroup> for u8 {
    fn from(group: AttributeGroup) -> Self {
        group as u8
    }
}

impl UnknownGroup {
    /// returns the underlying value for this UnknownGroup
    pub fn value(self) -> u8 {
        self.0
    }
}

impl From<AttributeGroup> for DelimiterTag {
    fn from(group: AttributeGroup) -> DelimiterTag {
        match group {
            AttributeGroup::OpAttrs => DelimiterTag::OperationAttributes,
            AttributeGroup::JobAttrs => DelimiterTag::JobAttributes,
            AttributeGroup::EndOfAttributes => DelimiterTag::EndOfAttributes,
            AttributeGroup::PrinterAttrs => DelimiterTag::PrinterAttributes,
            _ => DelimiterTag::UnsupportedAttributes,
        }
    }
}

impl From<DelimiterTag> for AttributeGroup {
    fn from(tag: DelimiterTag) -> AttributeGroup {
        match tag {
            DelimiterTag::OperationAttributes => AttributeGroup::OpAttrs,
            DelimiterTag::JobAttributes => AttributeGroup::JobAttrs,
            DelimiterTag::EndOfAttributes => AttributeGroup::EndOfAttributes,
            DelimiterTag::PrinterAttributes => AttributeGroup::PrinterAttrs,
            DelimiterTag::UnsupportedAttributes => AttributeGroup::Unsupported,
        }
    }
}

/// A unique key identifying an IPP attribute by group and name.
///
/// IPP attributes are scoped by their enclosing attribute group, identified
/// by a [`AttributeGroup`]. The same attribute name may legally appear in
/// multiple groups, making the `(group, key)` pair the true identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupKey {
    /// The IPP attribute group this attribute belongs to.
    pub group: AttributeGroup,
    /// The attribute's name within the group.
    pub key: String,
}

/// A flattened attribute map derived from [`IppAttributes`].
///
/// Internally, this stores cloned [`IppAttribute`] values keyed by [`GroupKey`].
/// It is intended for **read-heavy** access patterns where repeated attribute
/// lookups are required.
///
/// Construction preserves:
/// - original group boundaries
/// - original attribute values and metadata
#[derive(Debug, Clone)]
pub struct AttrMap {
    attrs: HashMap<GroupKey, IppAttribute>,
}

impl From<&IppAttributes> for AttrMap {
    /// Constructs an [`AttrMap`] from a reference to [`IppAttributes`].
    ///
    /// This conversion:
    /// - Iterates all attribute groups
    /// - Flattens attributes into a `(group, name)` map
    /// - Clones each [`IppAttribute`]
    ///
    /// This is intentionally implemented for `&IppAttributes` to avoid
    /// consuming the original structure.
    fn from(attrs: &IppAttributes) -> Self {
        let mut map = HashMap::new();

        for group in attrs.groups().iter() {
            for (name, attr) in group.attributes().iter() {
                let key = GroupKey {
                    group: group.tag().into(),
                    key: name.to_string(),
                };

                map.insert(key, attr.clone());
            }
        }

        AttrMap { attrs: map }
    }
}

/// Attempts to extract a borrowed string slice from an [`IppValue`].
///
/// This function normalizes all IPP string-like value variants into `&str`
/// where possible. Non-string values return `None`.
///
/// This helper is intentionally **non-recursive** and does not allocate.
#[inline]
fn match_str(v: &IppValue) -> Option<&str> {
    match v {
        IppValue::OctetString(v)
        | IppValue::TextWithoutLanguage(v)
        | IppValue::NameWithoutLanguage(v)
        | IppValue::Charset(v)
        | IppValue::NaturalLanguage(v)
        | IppValue::Uri(v)
        | IppValue::UriScheme(v)
        | IppValue::Keyword(v)
        | IppValue::MimeMediaType(v)
        | IppValue::MemberAttrName(v)
        | IppValue::NameWithLanguage { name: v, .. } => Some(v.as_str()),

        IppValue::TextWithLanguage { text, .. } => Some(text.as_str()),

        _ => None,
    }
}

impl AttrMap {
    /// Returns a reference to an attribute value by group and name.
    ///
    /// This performs an exact lookup using the provided [`AttributeGroup`]
    /// and attribute name.
    pub fn get<V: Into<AttributeGroup>>(&self, group: V, key: &str) -> Option<&IppValue> {
        let group = group.into();
        let reference = GroupKey {
            group,
            key: key.to_string(),
        };

        self.attrs.get(&reference).map(|v| v.value())
    }

    /// Returns a borrowed string value for an attribute, if possible.
    ///
    /// This succeeds only if the attribute value is a string-like IPP type.
    /// Array and collection values are **not** expanded.
    pub fn get_str<V: Into<AttributeGroup>>(&self, group: V, key: &str) -> Option<&str> {
        self.get(group, key).and_then(match_str)
    }

    /// Returns multiple string values for an attribute.
    ///
    /// Behavior:
    /// - `Array` → extracts each element as a string
    /// - single string → returns a one-element vector
    /// - any incompatible type → returns `None`
    ///
    /// This function performs **single-depth extraction only** and does not
    /// recursively flatten nested arrays.
    pub fn get_strings<V: Into<AttributeGroup>>(&self, group: V, key: &str) -> Option<Vec<&str>> {
        self.get(group, key).and_then(|v| match v {
            IppValue::Array(arr) => arr.iter().map(match_str).collect::<Option<Vec<&str>>>(),
            _ => match_str(v).map(|s| vec![s]),
        })
    }

    /// Returns a borrowed i32 value for an attribute, if possible.
    pub fn get_i32<V: Into<AttributeGroup>>(&self, group: V, key: &str) -> Option<i32> {
        self.get(group, key).and_then(|v| match v {
            IppValue::Integer(i) => Some(*i),
            _ => None,
        })
    }

    /// Returns a borrowed bool value for an attribute, if possible.
    pub fn get_bool<V: Into<AttributeGroup>>(&self, group: V, key: &str) -> Option<bool> {
        self.get(group, key).and_then(|v| match v {
            IppValue::Boolean(b) => Some(*b),
            _ => None,
        })
    }

    /// returns a localized string
    pub fn get_local_str<V: Into<AttributeGroup>>(
        &self,
        group: V,
        key: &str,
    ) -> Option<LocalizedString> {
        self.get(group, key).and_then(|v| match v {
            IppValue::TextWithLanguage { text, language } => Some(LocalizedString {
                value: text.to_string(),
                language: LangTag::from_str(language).ok(),
            }),
            IppValue::NameWithLanguage { name, language } => Some(LocalizedString {
                value: name.to_string(),
                language: LangTag::from_str(language).ok(),
            }),
            IppValue::TextWithoutLanguage(text) | IppValue::NameWithoutLanguage(text) => {
                Some(LocalizedString {
                    value: text.clone(),
                    language: None,
                })
            }
            _ => None,
        })
    }

    /// returns a datetime variant if type doesn't match it returns `None`
    pub fn get_datetime<V: Into<AttributeGroup>>(
        &self,
        group: V,
        key: &str,
    ) -> Option<IppDateTime> {
        self.get(group, key).and_then(|v| match v {
            IppValue::DateTime {
                year,
                month,
                day,
                hour,
                minutes,
                seconds,
                deci_seconds,
                utc_dir,
                utc_hours,
                utc_mins,
            } => Some(IppDateTime {
                year: *year,
                month: *month,
                day: *day,
                hour: *hour,
                minutes: *minutes,
                seconds: *seconds,
                deci_seconds: *deci_seconds,
                utc_dir: *utc_dir,
                utc_hours: *utc_hours,
                utc_mins: *utc_mins,
            }),
            _ => None,
        })
    }

    /// Returns a Unix epoch timestamp for a DateTime attribute, if possible.
    pub fn get_datetime_epoch<V: Into<AttributeGroup>>(&self, group: V, key: &str) -> Option<i64> {
        self.get_datetime(group, key).and_then(|dt| dt.into_epoch())
    }
}

/// Represents an IPP DateTime attribute value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct IppDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub deci_seconds: u8,
    pub utc_dir: char, // '+' or '-'
    pub utc_hours: u8, // offset hours
    pub utc_mins: u8,  // offset minutes
}

impl IppDateTime {
    /// converts to unix epoch (seconds since 1970 in UTC timezone)
    pub fn into_epoch(self) -> Option<i64> {
        let dt = self;
        // Build chrono::DateTime<Utc>
        let naive =
            chrono::NaiveDate::from_ymd_opt(dt.year as i32, dt.month as u32, dt.day as u32)?
                .and_hms_milli_opt(
                    dt.hour as u32,
                    dt.minutes as u32,
                    dt.seconds as u32,
                    dt.deci_seconds as u32 * 100,
                )?;

        // Apply UTC offset
        let offset_seconds = match dt.utc_dir {
            '+' => dt.utc_hours as i32 * 3600 + dt.utc_mins as i32 * 60,
            '-' => -(dt.utc_hours as i32 * 3600 + dt.utc_mins as i32 * 60),
            _ => 0,
        };

        Some(
            DateTime::<Utc>::from_naive_utc_and_offset(
                naive - chrono::Duration::seconds(offset_seconds as i64),
                Utc,
            )
            .timestamp(),
        )
    }
}

/// used to define a range of values for IPP
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IntegerRange {
    /// a collection of integer values
    Values(Vec<i32>),
    /// a (min, max) range of values.
    Range(i32, i32),
}