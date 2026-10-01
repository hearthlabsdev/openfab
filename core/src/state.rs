//! Printing state primitives shared across the OpenMPD ecosystem.
//!
//! This module defines **authoritative state enums** used to track:
//!
//! - Physical device availability
//! - Logical print job lifecycle
//! - Per-device execution attempts
//!
//! These types intentionally live in a **core module** so they can be
//! shared by:
//!
//! - Central coordination services
//! - Local print daemons
//! - IPP / non-IPP protocol frontends
//! - Accounting and policy engines
//!
//! ## Design Principles
//!
//! - **Layered state**: logical jobs are distinct from physical execution
//! - **Transport-agnostic**: usable with IPP, USB, serial, or virtual devices
//! - **Monotonic transitions**: states only move forward in normal operation
//! - **Auditable**: state changes are intended to be persisted
//!
//! ## Relationship Between State Types
//!
//! ```text
//! PrinterStatus      → describes a device
//! JobState           → describes user intent
//! ExecutionState     → describes a single execution attempt
//! ```
//!
//! A single `JobState` may result in **multiple** `ExecutionState` rows
//! due to retries, failures, or rescheduling.

use serde_derive::{Deserialize, Serialize};

/// High-level status of a **physical or virtual printer device**.
///
/// This reflects *device reachability and readiness*, not whether a
/// specific job is running.
///
/// ## Scope
///
/// - Owned by **device discovery / health monitoring**
/// - Updated by:
///   - mDNS / IPP discovery
///   - heartbeat checks
///   - local daemon health probes
///
/// ## Persistence
///
/// This value is typically stored on the printer record itself and
/// updated whenever new telemetry is received.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrinterStatus {
    /// Printer is reachable and able to accept jobs.
    ///
    /// This does **not** guarantee immediate availability; the printer
    /// may still be busy.
    Online,

    /// Printer is known but currently offline.
    ///
    /// Examples:
    /// - powered down
    /// - intentionally disconnected
    /// - temporarily unavailable
    Offline,

    /// Printer is reachable but currently occupied.
    ///
    /// Examples:
    /// - actively printing
    /// - warming up
    /// - performing maintenance
    Busy,

    /// Printer cannot be contacted via its configured transport.
    ///
    /// This indicates a **network or connectivity failure**, not
    /// necessarily a printer fault.
    Unreachable,

    /// Printer has entered an error state.
    ///
    /// Examples:
    /// - paper jam
    /// - filament runout
    /// - hardware fault
    ///
    /// Additional diagnostic information should be stored separately.
    Error,

    /// Printer has been permanently removed from service.
    ///
    /// - Not eligible for scheduling
    /// - Retained only for historical/audit purposes
    Retired,
}

/// High-level lifecycle state of a **logical print job**.
///
/// This represents **user intent**, independent of which printer
/// ultimately executes the job.
///
/// ## Scope
///
/// - Owned by the **job scheduler / coordinator**
/// - Stable and auditable
/// - Never tied directly to a single printer
///
/// ## Mapping
///
/// Roughly corresponds to IPP job states, but is intentionally
/// generalized to support non-IPP backends.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobState {
    /// Job has been created but not yet admitted for execution.
    ///
    /// Examples:
    /// - awaiting authorization (badge reader)
    /// - awaiting quota approval
    /// - awaiting file upload completion
    Pending,

    /// Job has been accepted and is eligible for execution.
    ///
    /// - Printer assignment may or may not have occurred
    /// - Job may be queued for a capability class
    Scheduled,

    /// At least one execution attempt is actively running.
    ///
    /// This state reflects **intent**, not execution success.
    Running,

    /// Job has completed successfully.
    ///
    /// - All required output was produced
    /// - Accounting has been finalized
    Completed,

    /// Job has failed permanently.
    ///
    /// - All retries exhausted
    /// - Manual intervention required
    Failed,

    /// Job was explicitly cancelled.
    ///
    /// - User initiated
    /// - Administrator initiated
    Cancelled,

    /// Job expired before execution.
    ///
    /// Examples:
    /// - user never authenticated
    /// - printer unavailable for too long
    /// - time-bound authorization elapsed
    Expired,
}

/// Low-level execution state of a **single job attempt on a specific printer**.
///
/// Each execution attempt corresponds to:
///
/// - one job
/// - one printer
/// - one scheduling decision
///
/// A single job may have **multiple execution records**.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionState {
    /// Execution has been created and queued for a printer.
    ///
    /// The printer has been selected, but no data transfer
    /// has started.
    Queued,

    /// Job data is being transmitted to the printer.
    ///
    /// Examples:
    /// - streaming raster data
    /// - uploading a file to a device API
    Dispatching,

    /// Printer has acknowledged execution and is actively printing.
    ///
    /// This state may last a long time for fabrication devices.
    Printing,

    /// Execution completed successfully.
    ///
    /// - Printer confirmed completion
    /// - Output was produced
    Succeeded,

    /// Execution failed.
    ///
    /// Examples:
    /// - transport failure
    /// - printer error
    /// - malformed job data
    Failed,

    /// Execution was aborted mid-run.
    ///
    /// Examples:
    /// - user cancellation
    /// - printer emergency stop
    /// - daemon shutdown
    Aborted,
}
