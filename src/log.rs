use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

/// Simple logger that writes to a file and stores messages in memory.
#[derive(Debug)]
pub struct Log {
    file: std::fs::File,
    buffer: Vec<String>,
}

impl Log {
    /// Create a new logger. Appends to the file if it already exists.
    pub fn new(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.into())?;
        Ok(Self {
            file,
            buffer: Vec::new(),
        })
    }

    /// Log a message (writes to file + stores in buffer).
    pub fn write(&mut self, msg: impl Into<String>) {

        let msg = msg.into();

        // Format timestamp
        let now = Local::now();
        let ts = now.format("%Y-%m-%d %H:%M:%S");

        // Write "timestamp | message"
        writeln!(self.file, "[{}] {}", ts, msg).unwrap();

        // Write to file
        self.file.flush().unwrap();

        // Keep in memory for later (e.g. UI display)
        self.buffer.push(msg);
    }

    /// Get all in-memory logs (for displaying in ratatui)
    pub fn entries(&self) -> &[String] {
        &self.buffer
    }
}
