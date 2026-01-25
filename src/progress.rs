//! Progress tracking and callback utilities
//!
//! This module provides types and utilities for tracking progress during
//! long-running compression, decompression, and archive operations.

/// Progress information for operations
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// Overall progress percentage (0.0 to 100.0)
    pub percentage: f32,
    /// Number of bytes processed so far
    pub bytes_processed: u64,
    /// Total number of bytes to process
    pub total_bytes: u64,
    /// Current processing stage
    pub stage: ProcessingStage,
    /// Optional human-readable status message
    pub message: Option<String>,
}

impl ProgressInfo {
    /// Create a new progress info
    pub fn new(stage: ProcessingStage, bytes_processed: u64, total_bytes: u64) -> Self {
        let percentage = if total_bytes > 0 {
            (bytes_processed as f32 / total_bytes as f32) * 100.0
        } else {
            100.0
        };

        Self {
            percentage: percentage.clamp(0.0, 100.0),
            bytes_processed,
            total_bytes,
            stage,
            message: None,
        }
    }

    /// Add a message to the progress info
    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());
        self
    }
}

/// Stages of processing for compression/decompression operations
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStage {
    /// Reading input file
    Reading,
    /// Computing Huffman frequencies
    ComputingFrequencies,
    /// Building Huffman tree
    BuildingTree,
    /// Encoding data with Huffman codes
    Encoding,
    /// Encrypting data (if password provided)
    Encrypting,
    /// Decrypting data (if encrypted)
    Decrypting,
    /// Decoding Huffman data
    Decoding,
    /// Writing output file
    Writing,
    /// Finalizing operation
    Finalizing,
}

impl fmt::Display for ProcessingStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingStage::Reading => write!(f, "Reading"),
            ProcessingStage::ComputingFrequencies => write!(f, "Computing frequencies"),
            ProcessingStage::BuildingTree => write!(f, "Building Huffman tree"),
            ProcessingStage::Encoding => write!(f, "Encoding"),
            ProcessingStage::Encrypting => write!(f, "Encrypting"),
            ProcessingStage::Decrypting => write!(f, "Decrypting"),
            ProcessingStage::Decoding => write!(f, "Decoding"),
            ProcessingStage::Writing => write!(f, "Writing"),
            ProcessingStage::Finalizing => write!(f, "Finalizing"),
        }
    }
}

use std::fmt;

/// Type alias for progress callback functions
pub type ProgressCallback = Box<dyn FnMut(&ProgressInfo) + Send + Sync>;

/// Builder for setting up progress callbacks
pub struct ProgressBuilder {
    callback: Option<ProgressCallback>,
}

impl std::fmt::Debug for ProgressBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressBuilder")
            .field("callback", &"<callback>")
            .finish()
    }
}

impl ProgressBuilder {
    /// Create a new progress builder
    pub fn new() -> Self {
        Self { callback: None }
    }

    /// Set a progress callback function
    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&ProgressInfo) + Send + Sync + 'static,
    {
        self.callback = Some(Box::new(callback));
        self
    }

    /// Build the progress callback
    pub fn build(self) -> Option<ProgressCallback> {
        self.callback
    }
}

impl Default for ProgressBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility struct for tracking progress internally
pub struct ProgressTracker {
    total_bytes: u64,
    bytes_processed: u64,
    stage: ProcessingStage,
    callback: Option<ProgressCallback>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(
        total_bytes: u64,
        stage: ProcessingStage,
        callback: Option<ProgressCallback>,
    ) -> Self {
        Self {
            total_bytes,
            bytes_processed: 0,
            stage,
            callback,
        }
    }

    /// Update progress with additional bytes processed
    pub fn update(&mut self, bytes: u64) {
        self.bytes_processed += bytes;
        self.notify();
    }

    /// Set a new processing stage
    pub fn set_stage(&mut self, stage: ProcessingStage) {
        self.stage = stage;
        self.notify();
    }

    /// Add bytes to the total (for multi-stage operations)
    pub fn add_total_bytes(&mut self, additional: u64) {
        self.total_bytes += additional;
        self.notify();
    }

    /// Send progress notification to callback
    pub fn notify(&mut self) {
        if let Some(ref mut callback) = self.callback {
            let info =
                ProgressInfo::new(self.stage.clone(), self.bytes_processed, self.total_bytes);
            callback(&info);
        }
    }

    /// Get current progress percentage
    pub fn percentage(&self) -> f32 {
        if self.total_bytes > 0 {
            (self.bytes_processed as f32 / self.total_bytes as f32) * 100.0
        } else {
            100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_info_creation() {
        let info = ProgressInfo::new(ProcessingStage::Encoding, 500, 1000);
        assert_eq!(info.percentage, 50.0);
        assert_eq!(info.bytes_processed, 500);
        assert_eq!(info.total_bytes, 1000);
        assert_eq!(info.stage, ProcessingStage::Encoding);
    }

    #[test]
    fn test_progress_info_clamping() {
        // Test percentage clamping
        let info = ProgressInfo::new(ProcessingStage::Encoding, 1500, 1000);
        assert_eq!(info.percentage, 100.0);

        let info = ProgressInfo::new(ProcessingStage::Encoding, 0, 1000);
        assert_eq!(info.percentage, 0.0);
    }

    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new(1000, ProcessingStage::Encoding, None);
        assert_eq!(tracker.percentage(), 0.0);

        tracker.update(500);
        assert_eq!(tracker.percentage(), 50.0);

        tracker.set_stage(ProcessingStage::Finalizing);
        // Should still maintain the same progress percentage
        assert_eq!(tracker.percentage(), 50.0);
    }

    #[test]
    fn test_progress_callback() {
        use std::sync::atomic::{AtomicU32, Ordering};

        static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

        let callback = Box::new(|_: &ProgressInfo| {
            CALL_COUNT.fetch_add(1, Ordering::SeqCst);
        });

        let mut tracker = ProgressTracker::new(100, ProcessingStage::Encoding, Some(callback));

        tracker.update(50);
        tracker.update(50);

        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 2); // 2 updates only
    }
}
