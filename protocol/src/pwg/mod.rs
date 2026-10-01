//! # Printer Working Group (PWG) Specifications
//!
//! This module groups functionality and data structures that implement or model
//! specifications published by the **Printer Working Group (PWG)**.
//!
//! The PWG defines standards that extend and refine the Internet Printing
//! Protocol (IPP), covering areas such as printer discovery, job management,
//! system services, accounting, notifications, and emerging domains like
//! additive manufacturing.
//!
//! ## Module Structure
//!
//! This module is organized into submodules that correspond to major PWG
//! specification families:
//!
//! * [`ipp3d`] — Implements **IPP 3D Printing Extensions**
//!   (PWG 5100.21), providing strongly typed representations of 3D job tickets,
//!   materials, build volumes, printer capabilities, and presets.
//!
//! * [`services`] — Implements **IPP System Service–related specifications**
//!   (e.g. PWG 5100.22), including system, job, and resource attributes that
//!   apply across one or more printers managed by a system.
//!
//! ## Design Philosophy
//!
//! * **Specification-aligned** — Types and attribute names follow PWG and RFC
//!   terminology closely to reduce ambiguity.
//! * **Structured-first** — PWG specifications rely heavily on IPP collections;
//!   this module favors structured Rust types that map cleanly to
//!   `IppValue::Collection`.
//! * **Composable** — Submodules are independent but interoperable, allowing
//!   applications to adopt only the parts they need.
//!
//! ## Standards Coverage
//!
//! The following standards are directly or indirectly represented here:
//!
//! * **RFC 8010 / RFC 8011** — IPP core model and semantics
//! * **PWG 5100.21** — IPP 3D Printing Extensions
//! * **PWG 5100.22** — IPP System Service
//! * **RFC 3995 / RFC 3996** — Event notifications and subscriptions
//!
//! Additional PWG or IETF extensions may be added as new submodules over time.
//!
//! ## Intended Audience
//!
//! This module is intended for:
//!
//! * IPP client and server implementers
//! * Print infrastructure and device management systems
//! * Additive manufacturing (3D printing) service developers
//! * Researchers and tooling authors working with PWG standards
//!
//! ## Notes
//!
//! * This crate focuses on **data modeling and encoding**, not transport-level
//!   concerns such as HTTP handling or TLS.
//! * Implementations SHOULD validate live device responses against the
//!   capabilities reported via IPP.
//!
//! ## References
//!
//! * Printer Working Group — https://www.pwg.org/
//! * IPP Registry — https://www.iana.org/assignments/ipp-registrations/
//!
//! ---
pub mod ipp3d;
pub mod services;
