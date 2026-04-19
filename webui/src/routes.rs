use crate::utils::Guard;
use rocket::serde::json::Json;
use rocket::{FromForm, delete, get, post, put, route::Route, routes};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    Discovery,
}

// a struct for filtering events
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventQuery {
    /// host name or IP address associated with the print server
    pub host: Option<String>,
    /// the self declared name of the printer
    pub printer: Option<String>,
    /// the type of event to filter for
    pub event_types: Option<Vec<EventType>>,
    /// whether the outcome of an event was successful
    pub succeeded: Option<bool>,
}
