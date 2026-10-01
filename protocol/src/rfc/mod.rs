//! # RFC-based IPP Modules
//!
//! This module provides implementations of **Internet Printing Protocol (IPP) features**
//! based on relevant RFCs and PWG specifications. The goal is to offer **type-safe,
//! protocol-compliant Rust abstractions** for handling accounting, event notification,
//! and subscription management in IPP environments.
//!
//! ## Submodules
//!
//! ### `accounting`
//!
//! The `accounting` submodule implements IPP accounting features, such as tracking
//! job usage, printer resource consumption, and cost information. This is generally
//! based on IPP usage and billing attributes defined in the IPP Base and related
//! PWG specifications.  
//! Key features include:
//! - Recording **job start/finish events** with associated accounting information.
//! - Storing and retrieving **job-based or printer-based usage statistics**.
//! - Preparing structured attributes for **IPP `job-accounting` requests**.
//!
//! ### `notifications`
//!
//! The `notifications` submodule handles **event notification and subscription**
//! in accordance with [RFC 3996](https://www.rfc-editor.org/rfc/rfc3996.html), which
//! specifies the IPP Event Notifications framework. It provides:
//! - Management of **subscriptions** to printer or job events.
//! - Sending **notifications** when subscribed events occur (e.g., job-completed,
//!   printer-error).
//! - Construction of **event attributes** and **subscription objects** as
//!   structured IPP attributes (`IppValue::Collection` / `BTreeMap`).
//! - Support for **event filtering, expiration, and delivery** as defined by RFC 3996.
//!
//! ## Design Considerations
//!
//! - All structured attributes use `BTreeMap<String, IppValue>` to ensure deterministic
//!   ordering for IPP transmission.
//! - Event notifications and accounting are designed to integrate seamlessly with
//!   standard IPP requests such as `Create-Job`, `Get-Printer-Attributes`, and
//!   `Send-Document`.
//! - Optional multilingual fields are represented via `LocalizedString` to support
//!   `textWithLanguage` attributes, ensuring RFC 8011 compliance.
//!
//! ## References
//!
//! - [RFC 8010: IPP/1.1 Model and Semantics](https://www.rfc-editor.org/rfc/rfc8010.html)
//! - [RFC 8011: IPP/1.1 Encoding and Transport](https://www.rfc-editor.org/rfc/rfc8011.html)
//! - [RFC 3996: IPP Event Notifications](https://www.rfc-editor.org/rfc/rfc3996.html)
//! - [PWG 5100.x Series: Printer Working Group IPP Extensions](https://ftp.pwg.org/pub/pwg/candidates/)

pub mod accounting;
/// handles event notification and subscription in accordance with RFC 3996
pub mod notifications;
pub mod reasons;
