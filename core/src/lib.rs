#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_imports)]
//! # openmpd — Open Maker Printer Daemon
//!
//! **Open Maker Printer Daemon (openmpd)** is a modular, standards-aware
//! daemon for **network-discoverable, policy-controlled printing and
//! fabrication services**.
//!
//! It is designed to support both **traditional document printing**
//! and **maker / fabrication workflows**, while remaining interoperable
//! with existing IPP-aware clients.
//!
//! ## Code Example
//!
//! ### Advertising a Printer
//!
//! ```no_run
//! use openmpd::advertise::{IppAdvertiser, IppBroadcastBuilder};
//! use openmpd::IppTxtRecords;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut advertiser = IppAdvertiser::new()?;
//!     // Build standard IPP TXT records
//!     let txt = IppTxtRecords::new()
//!         .rp("ipp/print")
//!         .ty("Open Maker Virtual Printer")
//!         .product("(OpenMPD Virtual Printer)")
//!         .pdl("application/pdf,image/pwg-raster")
//!         .note("Development / virtual printer");
//!
//!     IppBroadcastBuilder::new(
//!         "OpenMPD Printer",       // Instance name shown in UIs
//!         "openmpd.local.",        // Hostname (mDNS)
//!     )
//!     .records(txt)
//!     .secure(false)              // Advertise _ipp._tcp (not IPPS)
//!     // .port(631)               // Optional: defaults to 631
//!     .advertise(&mut advertiser)?;
//!
//!     // Keep the process alive so advertisements persist.
//!     // Services are automatically withdrawn when `advertiser` is dropped.
//!     std::thread::park();
//! }
//! ```
//!
//! ## Design Goals
//!
//! - Open, inspectable infrastructure
//! - Standards-first network interoperability
//! - Clear separation between *discovery*, *policy*, and *execution*
//! - Support for non-traditional “printers” (plotters, cutters, virtual devices)
//!
//! ## Current Capabilities
//!
//! At present, `openmpd` provides:
//!
//! - IPP / IPPS service advertisement via DNS-SD
//! - IPP / IPPS service discovery via mDNS
//! - Structured handling of IPP-related TXT records
//!
//! These capabilities form the **network presence layer** of the daemon.
//!
//! ## Planned / Intended Capabilities
//!
//! The crate is structured to grow into a full daemon supporting:
//!
//! - Lightweight virtual printers
//! - Policy-based job admission and routing
//! - Network and trust-domain isolation
//! - Non-CUPS IPP servers
//! - Fabrication endpoints (e.g. plotters, laser cutters, 3D printers)
//!
//! Importantly, *printing* is treated as a **generalized materialization
//! pipeline**, not just ink-on-paper.
//!
//! ## Scope Boundaries
//!
//! `openmpd` intentionally does **not** assume:
//!
//! - A specific document conversion pipeline
//! - A specific spooler architecture
//! - A monolithic daemon model
//!
//! Instead, it provides **composable building blocks** that can be embedded
//! in larger systems or deployed standalone.
//!
//! ## Standards & References
//!
//! - RFC 6763 — DNS-Based Service Discovery  
//!   https://www.rfc-editor.org/rfc/rfc6763
//!
//! - IPP Everywhere™ (Printer Working Group)  
//!   https://www.pwg.org/ipp/everywhere.html
//!
//! - CUPS Network Printing Documentation  
//!   https://www.cups.org/doc/network.html
//!
//! - Apple AirPrint Specification  
//!   https://developer.apple.com/bonjour/printing-specification/
//!
//! ## Modules
//!
//! - [`advertise`] — Advertise IPP/IPPS services via DNS-SD
//! - [`discovery`] — Discover IPP/IPPS services on the local network
//!
//! Additional modules may define policy engines, virtual devices,
//! execution backends, and isolation mechanisms.
//!
//! ## Architectural Note
//!
//! Historically, printing stacks have tightly coupled *discovery*,
//! *policy*, and *execution*. `openmpd` deliberately decouples these
//! concerns to make:
//!
//! - failures diagnosable
//! - systems auditable
//! - non-paper devices first-class citizens

// this is needed in ipputils to import both [`serde_derive::Serialize`] and [`serde::Serialize`]
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
pub mod advertise;
pub mod discovery;
pub mod ipputils;
pub mod utils;
pub mod pwg;
pub mod rfc;
pub mod state;
pub mod config;
pub mod errors;
/// used for JSON API types and handlers
/// this includes definitions for anything outside of the scope of IPP including: library, device management, user and group management, and policy management.
pub mod api;

#[cfg(feature = "oidc")]
pub mod oidc;

use std::collections::HashMap;
use std::net::IpAddr;

/// Representation of a discovered IPP printer via DNS-SD / mDNS.
///
/// This struct represents the *result of service discovery*, not
/// an IPP capability query. Fields are populated from DNS-SD
/// service records (`_ipp._tcp`, `_ipps._tcp`) and associated TXT
/// records.
///
/// ## Sources
///
/// - DNS-Based Service Discovery (RFC 6763)
///   https://www.rfc-editor.org/rfc/rfc6763
/// - IPP Everywhere™
///   https://www.pwg.org/ipp/everywhere.html
/// - CUPS DNS-SD printer discovery
///   https://www.cups.org/doc/network.html
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IppPrinter {
    /// mDNS service instance name
    pub name: String,

    /// Hostname advertised via mDNS (typically ends in `.local.`)
    pub host: String,

    /// TCP port for the IPP service (usually 631)
    pub port: u16,

    /// IPv4 / IPv6 addresses resolved from the service record
    pub addresses: Vec<IpAddr>,

    /// Raw DNS-SD TXT key/value pairs
    pub txt: IppTxtRecords,

    /// Whether this printer was advertised via `_ipps._tcp`
    pub secure: bool,

    /// Unix epoch timestamp (seconds) when this printer was discovered
    pub discovered_at: i64,
}

impl IppPrinter {
    /// returns the underlying TXT records for this [`IppPrinter`] instance.
    pub fn records(&self) -> &IppTxtRecords {
        &self.txt
    }
}

/// Builder for DNS-SD TXT records used when advertising IPP printers.
///
/// This type encapsulates **de-facto standard TXT keys** used by
/// IPP, CUPS, and AirPrint for printer discovery.
///
/// While DNS-SD itself is standardized, the *meaning* of IPP TXT
/// records is defined by a combination of PWG specifications,
/// CUPS conventions, and Apple AirPrint requirements.
///
/// ## Authoritative & De-Facto Sources
///
/// - **RFC 6763 — DNS-Based Service Discovery**
///   https://www.rfc-editor.org/rfc/rfc6763
///
/// - **IPP Everywhere™ (PWG)**
///   https://www.pwg.org/ipp/everywhere.html
///
/// - **CUPS Network & DNS-SD Documentation**
///   https://www.cups.org/doc/network.html
///
/// - **Apple AirPrint Programming Guide**
///   https://developer.apple.com/bonjour/printing-specification/
///
/// ## Practical Notes
///
/// - Some fields are technically optional but *required in practice*
///   for interoperability.
/// - Incorrect TXT records often result in printers being discovered
///   but silently failing to print.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IppTxtRecords {
    records: HashMap<String, String>,
}

impl FromIterator<(String, String)> for IppTxtRecords {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        let records = HashMap::from_iter(iter);
        IppTxtRecords { records }
    }
}

impl IppTxtRecords {
    /// Create an empty TXT record set.
    pub fn new() -> Self {
        Self::default()
    }

    /// returns the optional rp record for this set of IPP TXT records.
    pub fn path(&self) -> Option<&String> {
        self.records.get("rp")
    }

    /// Insert an arbitrary TXT key/value pair.
    ///
    /// This can be used for vendor extensions or experimental fields.
    ///
    /// ## DNS-SD Constraints
    ///
    /// - Keys are case-sensitive
    /// - Individual TXT entries must be ≤ 255 bytes
    ///
    /// See RFC 6763 §6.
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.records.insert(key.into(), value.into());
    }

    /// Sets the `rp` (resource path) TXT record.
    ///
    /// Defines the **relative HTTP path** to the IPP endpoint.
    ///
    /// Clients construct the full IPP URI as:
    ///
    /// ```text
    /// ipp://<hostname>:<port>/<rp>
    /// ```
    ///
    /// ### Examples
    ///
    /// | `rp` value        | Resulting URI                          |
    /// |-------------------|-----------------------------------------|
    /// | `ipp/print`       | `ipp://printer.local:631/ipp/print`     |
    /// | `printers/queue1` | `ipp://printer.local:631/printers/queue1` |
    ///
    /// ### Requirements (in practice)
    ///
    /// - Must **not** include a leading `/`
    /// - Must **not** include scheme, host, or port
    /// - Required by:
    ///   - CUPS
    ///   - macOS
    ///   - iOS / AirPrint
    ///
    /// ## Sources
    ///
    /// - CUPS DNS-SD Behavior
    ///   https://www.cups.org/doc/network.html
    /// - Apple AirPrint Specification
    ///   https://developer.apple.com/bonjour/printing-specification/
    pub fn rp(mut self, rp: impl Into<String>) -> Self {
        self.insert("rp", rp);
        self
    }

    /// Sets the `ty` (type / display name) TXT record.
    ///
    /// Provides a **human-readable printer name** shown in print dialogs.
    ///
    /// ### Behavior
    ///
    /// - Used as the primary display label by:
    ///   - macOS
    ///   - GNOME / GTK
    ///   - iOS
    /// - Falls back to the mDNS instance name if omitted
    ///
    /// ### Semantics
    ///
    /// - No protocol meaning
    /// - Does not affect routing or queue selection
    ///
    /// ## Sources
    ///
    /// - Apple AirPrint Specification
    ///   https://developer.apple.com/bonjour/printing-specification/
    /// - CUPS DNS-SD Conventions
    ///   https://www.cups.org/doc/network.html
    pub fn ty(mut self, name: impl Into<String>) -> Self {
        self.insert("ty", name);
        self
    }

    /// Sets the `product` TXT record.
    ///
    /// Describes the **printer make and model** or product identity.
    ///
    /// This field originates from legacy IEEE 1284 device ID strings
    /// and is preserved for compatibility.
    ///
    /// ### Format
    ///
    /// Conventionally wrapped in parentheses:
    ///
    /// ```text
    /// (HP LaserJet Pro M404dn)
    /// (OpenMaker Virtual Printer)
    /// ```
    ///
    /// ### Behavior
    ///
    /// - Used by some clients for driver matching
    /// - Displayed in advanced printer details
    ///
    /// ## Sources
    ///
    /// - CUPS Source & Documentation
    ///   https://www.cups.org/doc/spec-ipp.html
    /// - IEEE 1284 Device ID (historical context)
    ///   https://standards.ieee.org/standard/1284-2000.html
    pub fn product(mut self, product: impl Into<String>) -> Self {
        self.insert("product", product);
        self
    }

    /// Sets the `pdl` (Page Description Language) TXT record.
    ///
    /// Advertises the **document formats** accepted by the printer.
    ///
    /// ### Format
    ///
    /// Comma-separated MIME types:
    ///
    /// ```text
    /// application/pdf,image/pwg-raster,image/jpeg
    /// ```
    ///
    /// ### Common Values
    ///
    /// - `application/pdf`
    /// - `image/pwg-raster`
    /// - `image/urf` (AirPrint)
    /// - `image/jpeg`
    /// - `image/png`
    ///
    /// ### Importance
    ///
    /// This is one of the **most critical** TXT records.
    /// Incorrect values often cause printers to be hidden
    /// or rejected by clients.
    ///
    /// ## Sources
    ///
    /// - IPP Everywhere™
    ///   https://www.pwg.org/ipp/everywhere.html
    /// - Apple AirPrint Specification
    ///   https://developer.apple.com/bonjour/printing-specification/
    pub fn pdl(mut self, pdl: impl Into<String>) -> Self {
        self.insert("pdl", pdl);
        self
    }

    /// Sets the `note` TXT record.
    ///
    /// Provides a **free-form, user-visible description** of the printer.
    ///
    /// ### Typical Usage
    ///
    /// - Physical location (`"Room 203"`)
    /// - Functional description (`"PDF archival queue"`)
    ///
    /// ### Semantics
    ///
    /// - Display-only
    /// - No protocol or routing behavior
    ///
    /// ## Sources
    ///
    /// - CUPS DNS-SD Conventions
    ///   https://www.cups.org/doc/network.html
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.insert("note", note);
        self
    }

    /// Returns the underlying TXT record map.
    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.records
    }
}
