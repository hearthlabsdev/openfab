
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Represents the execution status of a single G-code line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStatus {
    Pending,   // Line has not yet been sent to the printer
    Success,   // Line executed successfully
    Error,     // Printer returned an error for this line
}

/// Represents a single line in a print job.
#[derive(Debug, Clone)]
pub struct PrintLine {
    pub line_number: usize, // Line number in the G-code file
    pub line: String,       // The G-code command
    pub status: LineStatus, // Current status of this line
}

/// Represents a full print job for a Marlin printer.
#[derive(Debug)]
pub struct PrintJob {
    pub lines: Vec<PrintLine>, // All lines in the job
    pub current_line: usize,   // Index of the next line to send
}

impl PrintJob {
    /// Creates a new PrintJob from raw G-code text
    pub fn new(gcode: &str) -> Self {
        let lines = gcode
            .lines()
            .enumerate()
            .map(|(i, l)| PrintLine {
                line_number: i,
                line: l.trim().to_string(),
                status: LineStatus::Pending,
            })
            .collect();
        PrintJob { lines, current_line: 0 }
    }

    /// Marks a line as succeeded
    pub fn mark_success(&mut self, index: usize) {
        if let Some(line) = self.lines.get_mut(index) {
            line.status = LineStatus::Success;
        }
    }

    /// Marks a line as failed
    pub fn mark_error(&mut self, index: usize) {
        if let Some(line) = self.lines.get_mut(index) {
            line.status = LineStatus::Error;
        }
    }

    /// Returns the next pending line index
    pub fn next_pending(&self) -> Option<usize> {
        self.lines.iter().position(|l| l.status == LineStatus::Pending)
    }
}

pub struct MarlinDriver {
    jobs: Arc<Mutex<HashMap<u64, PrintJob>>>,
}
/*
impl GuestDriver for MarlinDriver {
    fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn init(&self) -> Result<bool, String> {
        Ok(true)
    }

    fn shutdown(&self) -> Result<bool, String> {
        Ok(true)
    }

    fn startjob(&self, job: Job) -> Result<u64, String> {
        
        Ok(0)
    }

    fn stopjob(&self, job_id: u64) -> Result<bool, String> {
        Ok(true)
    }

    fn pausejob(&self, job_id: u64) -> Result<bool, String> {
        Ok(true)
    }
}*/
