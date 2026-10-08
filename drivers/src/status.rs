use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum State {
    Idle,
    Busy,
    Printing,
    Paused,
    Finished,
    Stopped,
    Error,
    Attention,
    Ready,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterStatus {
    bed_temp: f64,
    target_bed_temp: Option<f64>,
    nozzle_temp: f64,
    target_nozzle_temp: Option<f64>,
    chamber_temp: Option<f64>,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    /// estimate of time remaining
    pub time_remaining_secs: i32,
    /// progress percentage
    pub progress: i32,
}