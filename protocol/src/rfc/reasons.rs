//! This module is modeled off of RFC3380 which handles more specialized reasons for printer state.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// this acts as a kind of error variant enum for IPP
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrinterStateReason {
    /// No media available in required tray
    MediaEmpty,

    /// Media jam detected
    MediaJam,

    /// Media is low but not empty
    MediaLow,

    /// Material is needed for print to proceed
    MaterialNeeded,

    /// Printer paused by operator
    Paused,

    /// Printer transitioning to paused
    MovingToPaused,

    /// Printer is shutting down
    Shutdown,

    /// Cover is open
    CoverOpen,

    /// Marker (ink/toner) is low
    MarkerSupplyLow,

    /// Marker (ink/toner) is empty
    MarkerSupplyEmpty,

    /// Catch-all for future or vendor-defined reasons
    Unknown(String),
}

impl fmt::Display for PrinterStateReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PrinterStateReason::MediaEmpty => "media-empty",
            PrinterStateReason::MediaJam => "media-jam",
            PrinterStateReason::MediaLow => "media-low",
            PrinterStateReason::Paused => "paused",
            PrinterStateReason::MovingToPaused => "moving-to-paused",
            PrinterStateReason::Shutdown => "shutdown",
            PrinterStateReason::CoverOpen => "cover-open",
            PrinterStateReason::MarkerSupplyLow => "marker-supply-low",
            PrinterStateReason::MarkerSupplyEmpty => "marker-supply-empty",
            // incorrect material supplied to the printer
            PrinterStateReason::MaterialNeeded => "material-needed",
            PrinterStateReason::Unknown(v) => v.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for PrinterStateReason {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "media-empty" => Ok(PrinterStateReason::MediaEmpty),
            "media-jam" => Ok(PrinterStateReason::MediaJam),
            "media-low" => Ok(PrinterStateReason::MediaLow),
            "paused" => Ok(PrinterStateReason::Paused),
            "moving-to-paused" => Ok(PrinterStateReason::MovingToPaused),
            "shutdown" => Ok(PrinterStateReason::Shutdown),
            "cover-open" => Ok(PrinterStateReason::CoverOpen),
            "marker-supply-low" => Ok(PrinterStateReason::MarkerSupplyLow),
            "marker-supply-empty" => Ok(PrinterStateReason::MarkerSupplyEmpty),
            "material-needed" => Ok(PrinterStateReason::MaterialNeeded),
            other => Ok(PrinterStateReason::Unknown(other.to_string())),
        }
    }
}

impl Serialize for PrinterStateReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PrinterStateReason {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PrinterStateReason::from_str(&s).map_err(serde::de::Error::custom)
    }
}
