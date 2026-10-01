//! IPP printer discovery via DNS-SD / mDNS.
//!
//! This module provides mechanisms to **browse and resolve IPP and IPPS
//! services** advertised on the local network.
//!
//! Discovered services are represented as resolved endpoints with
//! addresses, ports, and raw TXT records.
//!
//! ## Responsibilities
//!
//! - Browse for `_ipp._tcp.local.` and `_ipps._tcp.local.` services
//! - Resolve hostnames, ports, and IP addresses
//! - Track service appearance and removal
//!
//! ## Non-Responsibilities
//!
//! - IPP capability queries (`Get-Printer-Attributes`)
//! - Protocol negotiation
//! - Job submission
//!
//! ## Standards & References
//!
//! - RFC 6763 — DNS-Based Service Discovery
//!   https://www.rfc-editor.org/rfc/rfc6763
//!
//! - IPP Everywhere™
//!   https://www.pwg.org/ipp/everywhere.html
//!
//! - CUPS Printer Discovery Behavior
//!   https://www.cups.org/doc/network.html
//!
//! - Apple AirPrint Discovery Rules
//!   https://developer.apple.com/bonjour/printing-specification/
//!
//! ## Threading Model
//!
//! - Browsing occurs on background threads
//! - Discovered printers are stored in shared state
//! - Callers receive snapshots of the current view
//!
//! ## Design Notes
//!
//! Discovery is inherently **eventual and ephemeral**.
//! Callers should treat results as hints, not guarantees, and
//! expect printers to appear and disappear over time.

pub mod verify;
use crate::IppPrinter;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use crate::utils::unix_now;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Errors that may occur while discovering IPP printers via DNS-SD.
///
/// This error type primarily represents failures originating
/// from the underlying mDNS implementation.
///
/// ## Sources
///
/// - DNS-Based Service Discovery (RFC 6763)
///   https://www.rfc-editor.org/rfc/rfc6763
///
/// - `mdns-sd` crate documentation
///   https://docs.rs/mdns-sd/
#[derive(Debug, Error)]
pub enum IppDiscoveryError {
    /// An error originating from the mDNS / DNS-SD layer.
    ///
    /// This may occur if:
    /// - the mDNS daemon cannot be started
    /// - browsing for services fails
    /// - multicast traffic is unavailable
    #[error("mDNS error: {0}")]
    Mdns(#[from] mdns_sd::Error),
    /// an underlying [`ipp::prelude::IppError`] from the IPP crate.
    #[error("ipp error: {0}")]
    IppErr(#[from] ipp::prelude::IppError),
    /// route could not be retrieved from the IPP endpoint.
    #[error("missing rp txt record aka route on print server")]
    MissingRoute,
    /// the url is invalid for this printer.
    #[error("invalid uri: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    /// missing an attribute that the printer was required to produce.
    #[error("missing attribute: {0}")]
    MissingAttribute(String),
}

/// Discovers IPP and IPPS printers using DNS-SD / mDNS.
///
/// This type listens for `_ipp._tcp.local.` and `_ipps._tcp.local.`
/// service announcements and maintains a live view of printers
/// currently visible on the local network.
///
/// ## Behavior
///
/// - Resolves service instances into concrete printer endpoints
/// - Tracks additions and removals
/// - Aggregates TXT records for higher-level capability parsing
///
/// ## Threading Model
///
/// - Service browsing occurs on background threads
/// - Discovered printers are stored in a shared map protected
///   by a mutex
///
/// ## Standards & References
///
/// - **RFC 6763 — DNS-Based Service Discovery**
///   https://www.rfc-editor.org/rfc/rfc6763
///
/// - **IPP Everywhere™ (PWG)**
///   https://www.pwg.org/ipp/everywhere.html
///
/// - **CUPS DNS-SD Printer Discovery**
///   https://www.cups.org/doc/network.html
///
/// - **Apple AirPrint Specification**
///   https://developer.apple.com/bonjour/printing-specification/
///
/// ## Notes
///
/// - This performs *discovery only*; no IPP protocol traffic
///   is generated.
/// - TXT record correctness is essential for clients to
///   successfully submit jobs after discovery.
#[allow(missing_debug_implementations)]
pub struct IppDiscovery {
    daemon: ServiceDaemon,
    printers: Arc<Mutex<HashMap<String, IppPrinter>>>,
}

impl IppDiscovery {
    /// Create a new IPP discovery instance.
    ///
    /// Initializes an mDNS daemon capable of browsing for
    /// IPP and IPPS services.
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
    pub fn new() -> Result<Self, IppDiscoveryError> {
        Ok(Self {
            daemon: ServiceDaemon::new()?,
            printers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Start browsing for IPP and IPPS printers.
    ///
    /// This begins listening for:
    ///
    /// - `_ipp._tcp.local.` (plain IPP)
    /// - `_ipps._tcp.local.` (IPP over TLS)
    ///
    /// Browsing continues asynchronously until the
    /// `IppDiscovery` instance is dropped.
    ///
    /// ## Behavior
    ///
    /// - Newly discovered printers are added to the internal map
    /// - Removed services are pruned automatically
    ///
    /// ## Sources
    ///
    /// - RFC 6763 §5 — Browsing for Services
    ///   https://www.rfc-editor.org/rfc/rfc6763#section-5
    ///
    /// - CUPS Service Types
    ///   https://www.cups.org/doc/network.html
    pub fn start(&self) -> Result<(), IppDiscoveryError> {
        self.browse("_ipp._tcp.local.", false)?;
        self.browse("_ipps._tcp.local.", true)?;
        Ok(())
    }

    /// Browse for a specific DNS-SD service type.
    ///
    /// This method is responsible for:
    /// - resolving service instances
    /// - extracting addresses and TXT records
    /// - tracking service removal
    ///
    /// Browsing occurs on a dedicated background thread.
    ///
    /// ## Parameters
    ///
    /// - `service`:
    ///   DNS-SD service type (e.g. `_ipp._tcp.local.`)
    /// - `secure`:
    ///   Indicates whether this service corresponds to IPPS
    ///
    /// ## Sources
    ///
    /// - RFC 6763 §7 — Service Resolution
    ///   https://www.rfc-editor.org/rfc/rfc6763#section-7
    fn browse(&self, service: &str, secure: bool) -> Result<(), IppDiscoveryError> {
        let receiver = self.daemon.browse(service)?;
        let printers = self.printers.clone();

        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let name = info.get_fullname().to_string();

                        let printer = IppPrinter {
                            name: info.get_fullname().to_string(),
                            host: info.get_hostname().to_string(),
                            port: info.get_port(),
                            addresses: info
                                .get_addresses()
                                .iter()
                                .map(|v| v.to_ip_addr())
                                .collect(),
                            txt: info
                                .get_properties()
                                .iter()
                                .map(|p| (p.key().to_string(), p.val_str().to_string()))
                                .collect(),
                            secure,
                            discovered_at: unix_now(),
                        };

                        printers.lock().unwrap().insert(name, printer);
                    }

                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        printers.lock().unwrap().remove(&fullname);
                    }

                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Returns a snapshot of currently known printers.
    ///
    /// The returned list represents the state of discovery
    /// at the moment of the call.
    ///
    /// ## Behavior
    ///
    /// - This is a **copy** of the internal state
    /// - Discovery continues asynchronously in the background
    ///
    /// ## Notes
    ///
    /// - Printers may appear or disappear between calls
    /// - Higher-level code should treat results as ephemeral
    pub fn printers(&self) -> Vec<IppPrinter> {
        self.printers.lock().unwrap().values().cloned().collect()
    }
}
