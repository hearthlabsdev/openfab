#![deny(unused)]
//! This crate provides core utilities, as well as database, and JSON models used by the API server, and web UI.
//! I have yet to choose if this crate will also be used for the SDK or not.

/// module for dealing with user accounts.
pub mod accounts;
/// administrative user, group, api key, and access code management.
pub mod admin;
/// a module that contains primary error types used throughout this project
/// NOTE: this includes errors used by the web UI and the API server not underlying driver errors.
pub mod errors;
/// module for dealing with asset handling
pub mod library;
/// models for interacting with prints including queuing, print status, etc.
pub mod prints;
/// tools to make development easier
pub mod utils;
