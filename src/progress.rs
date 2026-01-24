use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

/// Progress tracking configuration
pub struct ProgressConfig {
    pub show_progress: bool,
    pub operation: OperationType,
}

#[derive(Debug, Clone)]
pub enum OperationType {
    Compression(String),
    Decompression(String),
    Encryption(String),
    Decryption(String),
    ArchiveCreation(String),
    ArchiveExtraction(String),
}

/// Progress tracker with ETA calculation
pub struct ProgressTracker {
    progress_bar: Option<ProgressBar>,
    start_time: Instant,
    config: ProgressConfig,
    total_bytes: u64,
    processed_bytes: u64,
}

impl ProgressTracker {
    pub fn new(config: ProgressConfig, total_bytes: u64) -> Self {
        let progress_bar = if config.show_progress {
            Some(create_progress_bar(&config.operation, total_bytes))
        } else {
            None
        };

        Self {
            progress_bar,
            start_time: Instant::now(),
            config,
            total_bytes,
            processed_bytes: 0,
        }
    }

    pub fn update(&mut self, bytes_processed: u64) -> io::Result<()> {
        self.processed_bytes += bytes_processed;

        if let Some(ref mut pb) = self.progress_bar {
            pb.set_position(self.processed_bytes);

            // Update ETA and rate every 100MB or every second
            if self.processed_bytes % (100 * 1024 * 1024) == 0
                || self.start_time.elapsed() >= Duration::from_secs(1)
            {
                let elapsed = self.start_time.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    let rate = self.processed_bytes as f64 / elapsed / (1024.0 * 1024.0); // MB/s
                    let remaining = if self.processed_bytes > 0 {
                        (self.total_bytes - self.processed_bytes) as f64
                            / (self.processed_bytes as f64 / elapsed) // seconds
                    } else {
                        0.0
                    };

                    pb.set_message(format!("{:.1} MB/s, ETA: {:.0}s", rate, remaining));
                }
            }
        }

        Ok(())
    }

    pub fn finish(&mut self) {
        if let Some(ref pb) = self.progress_bar {
            let elapsed = self.start_time.elapsed();
            let rate = if elapsed.as_secs_f64() > 0.0 {
                self.processed_bytes as f64 / elapsed.as_secs_f64() / (1024.0 * 1024.0)
            } else {
                0.0
            };

            pb.finish_with_message(format!(
                "Completed in {:.1}s ({:.1} MB/s)",
                elapsed.as_secs_f64(),
                rate
            ));
        }
    }

    pub fn set_message(&mut self, message: &str) {
        if let Some(ref pb) = self.progress_bar {
            pb.set_message(message.to_string());
        }
    }
}

impl Drop for ProgressTracker {
    fn drop(&mut self) {
        if let Some(ref pb) = self.progress_bar {
            if !pb.is_finished() {
                pb.abandon();
            }
        }
    }
}

/// Create a styled progress bar based on operation type
fn create_progress_bar(operation: &OperationType, total_bytes: u64) -> ProgressBar {
    let pb = ProgressBar::new(total_bytes);

    let msg = match operation {
        OperationType::Compression(filename) => format!("Compressing {}...", filename),
        OperationType::Decompression(filename) => format!("Decompressing {}...", filename),
        OperationType::Encryption(filename) => format!("Encrypting {}...", filename),
        OperationType::Decryption(filename) => format!("Decrypting {}...", filename),
        OperationType::ArchiveCreation(filename) => format!("Creating archive {}...", filename),
        OperationType::ArchiveExtraction(filename) => format!("Extracting archive {}...", filename),
    };

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} {msg} [{bar:40.cyan/blue}] {pos}/{len} bytes ({percent}%)")
            .expect("Invalid progress template")
            .progress_chars("##-"),
    );

    pb.set_message(msg);
    pb
}

/// Progress-aware wrapper for Read operations
pub struct ProgressReader<R: Read> {
    inner: R,
    tracker: Option<ProgressTracker>,
}

impl<R: Read> ProgressReader<R> {
    pub fn new(reader: R, tracker: Option<ProgressTracker>) -> Self {
        Self {
            inner: reader,
            tracker,
        }
    }
}

impl<R: Read> Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;

        if let Some(ref mut tracker) = self.tracker {
            tracker.update(bytes_read as u64)?;
        }

        Ok(bytes_read)
    }
}

/// Progress-aware wrapper for Write operations  
pub struct ProgressWriter<W: Write> {
    inner: W,
    tracker: Option<ProgressTracker>,
}

impl<W: Write> ProgressWriter<W> {
    pub fn new(writer: W, tracker: Option<ProgressTracker>) -> Self {
        Self {
            inner: writer,
            tracker,
        }
    }
}

impl<W: Write> Write for ProgressWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes_written = self.inner.write(buf)?;

        if let Some(ref mut tracker) = self.tracker {
            tracker.update(bytes_written as u64)?;
        }

        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
