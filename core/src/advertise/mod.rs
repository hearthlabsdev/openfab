//! IPP service advertisement via DNS-SD / mDNS.
//!
//! This module provides the primitives required to **publish IPP and IPPS
//! printer services** on a local network so they can be discovered by
//! clients such as:
//!
//! - CUPS
//! - macOS / iOS print dialogs
//! - Linux desktop environments
//!
//! ## Responsibilities
//!
//! - Register `_ipp._tcp.local.` and `_ipps._tcp.local.` services
//! - Attach DNS-SD TXT records describing printer capabilities
//! - Manage service lifetimes
//!
//! ## Non-Responsibilities
//!
//! - IPP protocol handling
//! - TLS configuration
//! - Job processing or spooling
//!
//! ## Standards & References
//!
//! - RFC 6763 — DNS-Based Service Discovery
//!   https://www.rfc-editor.org/rfc/rfc6763
//!
//! - IPP Everywhere™
//!   https://www.pwg.org/ipp/everywhere.html
//!
//! - CUPS DNS-SD Conventions
//!   https://www.cups.org/doc/network.html
//!
//! - Apple AirPrint Advertising Rules
//!   https://developer.apple.com/bonjour/printing-specification/
//!
//! ## Practical Notes
//!
//! - TXT record correctness is critical for interoperability
//! - Services are automatically unpublished when dropped
//! - Multiple queues may be advertised from a single process

mod builder;
pub use builder::IppBroadcastBuilder;

use crate::IppTxtRecords;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use thiserror::Error;

/// Errors that may occur while advertising an IPP service via DNS-SD.
///
/// This error type primarily wraps failures from the underlying
/// mDNS implementation.
///
/// ## Sources
///
/// - DNS-Based Service Discovery (RFC 6763)
///   https://www.rfc-editor.org/rfc/rfc6763
///
/// - `mdns-sd` crate documentation
///   https://docs.rs/mdns-sd/
#[derive(Debug, Error)]
pub enum IppAdvertiseError {
    /// An error originating from the mDNS / DNS-SD layer.
    ///
    /// This may occur if:
    /// - the mDNS daemon cannot be started
    /// - service registration fails
    /// - the network stack rejects multicast traffic
    #[error("mDNS error: {0}")]
    Mdns(#[from] mdns_sd::Error),
}

/// Advertises IPP and IPPS services using DNS-SD / mDNS.
///
/// This type manages the lifetime of one or more advertised
/// printer services. Services are **automatically unpublished**
/// when this value is dropped.
///
/// ## Behavior
///
/// - Advertises `_ipp._tcp.local.` and/or `_ipps._tcp.local.` services
/// - Registers DNS-SD TXT records describing printer capabilities
/// - Maintains ownership of `ServiceInfo` handles for proper teardown
///
/// ## Standards & References
///
/// - **RFC 6763 — DNS-Based Service Discovery**
///   https://www.rfc-editor.org/rfc/rfc6763
///
/// - **IPP Everywhere™ (PWG)**
///   https://www.pwg.org/ipp/everywhere.html
///
/// - **CUPS Network Printing & DNS-SD**
///   https://www.cups.org/doc/network.html
///
/// - **Apple AirPrint Specification**
///   https://developer.apple.com/bonjour/printing-specification/
///
/// ## Notes
///
/// - This type is transport-only; it does **not** implement the IPP protocol.
/// - It is valid to advertise multiple services (queues) from a single daemon.
/// - TXT record correctness is critical for client compatibility.
#[allow(missing_debug_implementations)]
pub struct IppAdvertiser {
    daemon: ServiceDaemon,
    services: Vec<ServiceInfo>,
}

impl IppAdvertiser {
    /// Create a new IPP advertiser.
    ///
    /// This initializes an mDNS service daemon used to register
    /// one or more IPP/IPPS services.
    ///
    /// ## Errors
    ///
    /// Returns an error if the underlying mDNS daemon
    /// cannot be created.
    ///
    /// ## Sources
    ///
    /// - `mdns-sd::ServiceDaemon`
    ///   https://docs.rs/mdns-sd/latest/mdns_sd/struct.ServiceDaemon.html
    pub fn new() -> Result<Self, IppAdvertiseError> {
        Ok(Self {
            daemon: ServiceDaemon::new()?,
            services: Vec::new(),
        })
    }

    /// Advertise an IPP or IPPS service via DNS-SD.
    ///
    /// Registers a printer service instance using either
    /// `_ipp._tcp.local.` or `_ipps._tcp.local.` and associates
    /// it with the provided TXT records.
    ///
    /// ## Parameters
    ///
    /// - `instance_name`:
    ///   Human-readable service instance name (shown in discovery UIs).
    /// - `hostname`:
    ///   Hostname where the IPP server is reachable
    ///   (typically ends with `.local.`).
    /// - `port`:
    ///   TCP port for the IPP service (commonly `631`).
    /// - `txt`:
    ///   DNS-SD TXT records describing printer capabilities.
    /// - `secure`:
    ///   If `true`, advertises `_ipps._tcp.local.` (TLS).
    ///
    /// ## Behavior
    ///
    /// - Clients discover the service via mDNS
    /// - The full IPP URI is constructed using:
    ///   - service type (`ipp` vs `ipps`)
    ///   - hostname
    ///   - port
    ///   - `rp` TXT record
    ///
    /// ## Errors
    ///
    /// Returns an error if:
    /// - service registration fails
    /// - invalid parameters are supplied to the mDNS layer
    ///
    /// ## Sources
    ///
    /// - RFC 6763 §7 — Service Instances
    ///   https://www.rfc-editor.org/rfc/rfc6763#section-7
    ///
    /// - CUPS DNS-SD Service Types
    ///   https://www.cups.org/doc/network.html
    ///
    /// - Apple AirPrint Service Advertising
    ///   https://developer.apple.com/bonjour/printing-specification/
    pub fn advertise(
        &mut self,
        instance_name: &str,
        hostname: &str,
        port: u16,
        txt: &IppTxtRecords,
        secure: bool,
    ) -> Result<(), IppAdvertiseError> {
        let service_type = if secure {
            "_ipps._tcp.local."
        } else {
            "_ipp._tcp.local."
        };

        let info = ServiceInfo::new(
            service_type,
            instance_name,
            hostname,
            "0.0.0.0",
            port,
            txt.as_map().clone(),
        )?;

        self.daemon.register(info.clone())?;
        self.services.push(info);

        Ok(())
    }

    /// Explicitly unpublish all advertised services.
    ///
    /// Dropping this value automatically unregisters services,
    /// so calling this method is optional.
    ///
    /// ## Behavior
    ///
    /// - All registered services are withdrawn from mDNS
    /// - No further advertisements will be made
    ///
    /// ## Sources
    ///
    /// - RFC 6763 §10 — Service Removal
    ///   https://www.rfc-editor.org/rfc/rfc6763#section-10
    pub fn shutdown(self) {
        // drop(self) unregisters services
    }
}
