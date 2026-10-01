//! Helpful utility functions.

/// produces the current unix epoch (seconds since 1970 UTC) this is useful for standardiing time accross timezones.
pub fn unix_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is before unix epoch")
        .as_secs() as i64
}