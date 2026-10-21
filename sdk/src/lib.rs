#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]
//! # OpenFAB Client
//!
//! A lightweight client library for interacting with the OpenFAB API.
//!
//! This crate provides the core request configuration used by generated or
//! hand-written API client code, including:
//!
//! - a configurable base URL
//! - optional custom HTTP headers
//! - a shared `reqwest::Client`
//! - bearer token authentication
//!
//! ## Example
//!
//! ```rust
//! use openfab_sdk::Configuration;
//!
//! let config = Configuration::new("my-access-token")
//!     .user_agent("OpenFAB CLI");
//! ```
//!
//! ## Configuration
//!
//! `Configuration` is the central type for client behavior. It stores the
//! connection base path, optional user-agent metadata, the HTTP client, and
//! the bearer token used for authenticated requests.
//!
//! A typical pattern is:
//!
//! ```rust
//! use openfab_sdk::Configuration;
//!
//! let config = Configuration::new("token")
//!     .default_user_agent();
//! ```
//!
//! This makes it easy to build API clients that share one configuration while
//! still allowing per-client overrides for things like hostnames, headers, or
//! token sources.
//!
//! ## Notes
//!
//! The default base path is `http://localhost`, which is useful for local
//! development and testing. In production or staging environments, you can
//! override it via the `base_path` field or by adding a dedicated configuration
//! builder in the future.
//!
//! This module intentionally keeps the HTTP transport concerns separate from
//! endpoint logic so higher-level API clients can stay focused on their
//! domain-specific operations.

extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate serde_repr;
extern crate url;

pub mod apis;
pub mod errors;
/// this module provides file validation and more comprehensive file type inference.
pub mod files;
pub mod models;
pub use apis::configuration::Configuration;
