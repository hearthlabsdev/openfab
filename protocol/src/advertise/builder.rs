//! Builder for constructing and advertising IPP/IPPS printer services.
//!
//! This type provides a **high-level, safe API** for assembling all
//! components required to advertise a printer via DNS-SD, while
//! enforcing best practices and interoperability constraints.
//!
//! It wraps [`IppTxtRecords`] and feeds into [`IppAdvertiser`].
//!
//! ## Responsibilities
//!
//! - Collect service identity (instance name, host, port)
//! - Construct DNS-SD TXT records using known IPP conventions
//! - Select IPP vs IPPS advertisement
//! - Validate required fields before broadcast
//!
//! ## Non-Responsibilities
//!
//! - IPP protocol handling
//! - TLS setup
//! - Job execution or policy enforcement
//!
//! ## Intended Usage
//!
//! ```rust,no_run
//! let mut advertiser = IppAdvertiser::new()?;
//!
//! IppBroadcastBuilder::new("OpenMaker PDF Printer")
//!     .hostname("printer.local.")
//!     .port(631)
//!     .rp("ipp/print")
//!     .ty("OpenMaker Virtual Printer")
//!     .product("(OpenMaker PDF Backend)")
//!     .pdl("application/pdf")
//!     .secure(false)
//!     .broadcast(&mut advertiser)?;
//! ```
//!
//! ## Standards & References
//!
//! - RFC 6763 — DNS-Based Service Discovery
//!   https://www.rfc-editor.org/rfc/rfc6763
//! - IPP Everywhere™
//!   https://www.pwg.org/ipp/everywhere.html
//! - CUPS DNS-SD Conventions
//!   https://www.cups.org/doc/network.html
//! - Apple AirPrint Advertising Rules
//!   https://developer.apple.com/bonjour/printing-specification/

use crate::{
    IppTxtRecords,
    advertise::{IppAdvertiseError, IppAdvertiser},
};

/// Builder for advertising a single IPP or IPPS printer service.
///
/// This builder enforces the presence of **practically required**
/// DNS-SD fields while allowing controlled extensibility.
///
/// Instances of this type are cheap and disposable.
#[derive(Debug, Default)]
pub struct IppBroadcastBuilder {
    instance_name: String,
    hostname: String,
    port: u16,
    secure: bool,
    txt: IppTxtRecords,
}

impl IppBroadcastBuilder {
    /// Create a new broadcast builder with a required service instance name.
    ///
    /// The instance name is the **human-visible name** shown in
    /// printer discovery dialogs.
    ///
    /// ## Examples
    ///
    /// - `"Office LaserJet"`
    /// - `"OpenMaker Virtual PDF Printer"`
    ///
    /// ## Notes
    ///
    /// - This is *not* the hostname
    /// - It must be unique per service type on the local network
    ///
    /// This method supplies the default value 631 for port in keeping with cups, but a custom port can be specified using the [`port`] method.
    pub fn new(instance_name: impl Into<String>, hostname: impl Into<String>) -> Self {
        Self {
            instance_name: instance_name.into(),
            hostname: hostname.into(),
            port: 631,
            secure: false,
            txt: IppTxtRecords::new(),
        }
    }

    /// Set the hostname where the IPP service is reachable.
    ///
    /// This is typically a `.local.` hostname resolvable via mDNS.
    ///
    /// ## Examples
    ///
    /// - `"printer.local."`
    /// - `"openmpd.local."`
    ///
    /// ## Requirements
    ///
    /// - Must resolve on the local network
    /// - Should end in `.local.` for mDNS
    pub fn hostname(mut self, hostname: impl Into<String>) -> Self {
        self.hostname = hostname.into();
        self
    }

    /// Set the TCP port for the IPP service.
    ///
    /// Common values:
    /// - `631` (standard IPP)
    /// - custom ports for isolated or virtual printers
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// overrite existing records
    pub fn records(mut self, txt: IppTxtRecords) -> Self {
        self.txt = txt;
        self
    }

    /// Set whether this service should be advertised as IPPS.
    ///
    /// - `false` → `_ipp._tcp.local.`
    /// - `true`  → `_ipps._tcp.local.`
    ///
    /// ## Notes
    ///
    /// - This only affects advertisement
    /// - TLS configuration is handled elsewhere
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Set the required `rp` (resource path) TXT record.
    ///
    /// This defines the **IPP endpoint path** on the host.
    ///
    /// ## Examples
    ///
    /// - `"ipp/print"`
    /// - `"printers/queue1"`
    ///
    /// ## Required
    ///
    /// This field is **mandatory for interoperability** and must be set
    /// before broadcasting.
    pub fn rp(mut self, rp: impl Into<String>) -> Self {
        self.txt = self.txt.rp(rp);
        self
    }

    /// Set the `ty` (type / display name) TXT record.
    pub fn ty(mut self, ty: impl Into<String>) -> Self {
        self.txt = self.txt.ty(ty);
        self
    }

    /// Set the `product` TXT record.
    pub fn product(mut self, product: impl Into<String>) -> Self {
        self.txt = self.txt.product(product);
        self
    }

    /// Set the `pdl` (Page Description Language) TXT record.
    pub fn pdl(mut self, pdl: impl Into<String>) -> Self {
        self.txt = self.txt.pdl(pdl);
        self
    }

    /// Set the `note` TXT record.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.txt = self.txt.note(note);
        self
    }

    /// Insert an arbitrary TXT record.
    ///
    /// This may be used for:
    /// - experimental fields
    /// - vendor extensions
    ///
    /// ## Warning
    ///
    /// Incorrect or non-standard TXT records may cause
    /// clients to ignore or mis-handle the printer.
    pub fn txt(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.txt.insert(key, value);
        self
    }

    /// Perform the actual DNS-SD advertisement.
    ///
    /// This consumes the builder and registers the service
    /// with the provided [`IppAdvertiser`].
    ///
    /// ## Errors
    ///
    /// Returns an error if:
    /// - required fields are missing
    /// - the mDNS layer rejects the service
    pub fn broadcast(self, advertiser: &mut IppAdvertiser) -> Result<(), IppAdvertiseError> {
        let hostname = self.hostname;
        let port = self.port;

        advertiser.advertise(&self.instance_name, &hostname, port, &self.txt, self.secure)
    }
}
