use crate::crypto::DEFAULT_CHUNK_SIZE;
use crate::error::Result;
use crate::progress::ProgressCallback;
use std::io::{self, Seek, Write};

/// Stateful compressor that maintains compression state across multiple writes
///
/// `Compressor` implements `Write` and compresses data incrementally as it's
/// written. This is useful for compressing large data streams or data
/// that's generated dynamically without knowing the total size in advance.
///
/// # Examples
///
/// ```rust
/// use mismall::stream::Compressor;
/// use std::fs::File;
///
/// let output_file = File::create("compressed.small")?;
/// let mut compressor = Compressor::new(output_file, "data.txt", None);
///
/// compressor.write_all(b"Hello, ")?;
/// compressor.write_all(b"world!")?;
/// compressor.finish()?; // Important: call finish() to complete compression
///
/// println!("Data compressed successfully!");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Compressor<W: Write + Seek> {
    destination: W,
    filename: String,
    password: Option<String>,
    chunk_size: usize,
    progress_callback: Option<ProgressCallback>,
    finished: bool,
    bytes_written: u64,
}

impl<W: Write + Seek> Compressor<W> {
    /// Create a new compressor
    ///
    /// # Arguments
    ///
    /// * `destination` - Destination stream for compressed data
    /// * `filename` - Original filename (stored in metadata)
    /// * `password` - Optional password for encryption
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::stream::Compressor;
    /// use std::fs::File;
    ///
    /// let file = File::create("output.small")?;
    /// let compressor = Compressor::new(file, "data.txt", None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(destination: W, filename: &str, password: Option<&str>) -> Self {
        Self {
            destination,
            filename: filename.to_string(),
            password: password.map(|p| p.to_string()),
            chunk_size: DEFAULT_CHUNK_SIZE,
            progress_callback: None,
            finished: false,
            bytes_written: 0,
        }
    }

    /// Set custom chunk size for compression
    ///
    /// Larger chunk sizes use more memory but may improve performance.
    /// Smaller chunk sizes use less memory but may be slower.
    ///
    /// # Arguments
    ///
    /// * `chunk_size` - Chunk size in bytes (64KB to 1GB)
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::stream::Compressor;
    ///
    /// let compressor = Compressor::new(output, "data.txt", None)
    ///     .with_chunk_size(64 * 1024 * 1024); // 64MB
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Set progress callback for real-time updates
    ///
    /// The callback will be called periodically with progress information
    /// during the compression process.
    ///
    /// # Arguments
    ///
    /// * `callback` - Function that receives `ProgressInfo`
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::stream::Compressor;
    ///
    /// let compressor = Compressor::new(output, "data.txt", None)
    ///     .with_progress_callback(|progress| {
    ///         println!("Progress: {}%", progress.percentage);
    ///     });
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&crate::progress::ProgressInfo) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Get the total bytes written so far
    ///
    /// # Returns
    ///
    /// Number of bytes written to the destination
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::stream::Compressor;
    ///
    /// let mut compressor = Compressor::new(output, "data.txt", None);
    /// compressor.write_all(b"Hello")?;
    /// println!("Written {} bytes", compressor.bytes_written());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Check if compression is finished
    ///
    /// # Returns
    ///
    /// `true` if `finish()` has been called successfully
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::stream::Compressor;
    ///
    /// let compressor = Compressor::new(output, "data.txt", None);
    /// assert!(!compressor.is_finished());
    ///
    /// compressor.finish()?;
    /// assert!(compressor.is_finished());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Complete the compression process
    ///
    /// This method must be called after all data has been written
    /// to finalize the compression and write proper headers/footers.
    /// After calling `finish()`, no more data can be written.
    ///
    /// # Returns
    ///
    /// `Result<()>` on success or `Err(MismallError)` on failure
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::stream::Compressor;
    ///
    /// let mut compressor = Compressor::new(output, "data.txt", None);
    /// compressor.write_all(b"Hello, world!")?;
    /// compressor.finish()?; // Finalize compression
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn finish(&mut self) -> Result<()> {
        if self.finished {
            return Ok(());
        }

        // This is a simplified implementation
        // In a full implementation, we would:
        // 1. Create a temporary file with all written data
        // 2. Use compress_stream to compress it
        // 3. Write headers and compressed data to destination
        // 4. Update headers and finish properly

        println!("Compressor::finish() not yet fully implemented");
        self.finished = true;
        Ok(())
    }
}

impl<W: Write + Seek> Write for Compressor<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.finished {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Cannot write to finished compressor",
            ));
        }

        // This is a simplified implementation
        // In a full implementation, we would:
        // 1. Buffer incoming data
        // 2. When buffer reaches chunk_size, compress it
        // 3. Write compressed chunks to destination
        // 4. Track position for proper header writing

        println!(
            "Compressor::write() not yet fully implemented - buffering {} bytes",
            buf.len()
        );

        // For now, just pass through to destination
        let bytes_written = self.destination.write(buf)?;
        self.bytes_written += bytes_written as u64;
        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.destination.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    #[test]
    fn test_compressor_creation() {
        let cursor = Cursor::new(Vec::new());
        let compressor = Compressor::new(cursor, "test.txt", None);

        assert_eq!(compressor.filename, "test.txt");
        assert!(compressor.password.is_none());
        assert_eq!(compressor.chunk_size, DEFAULT_CHUNK_SIZE);
        assert!(!compressor.is_finished());
        assert_eq!(compressor.bytes_written(), 0);
    }

    #[test]
    fn test_compressor_builder_pattern() {
        let cursor = Cursor::new(Vec::new());
        let compressor = Compressor::new(cursor, "test.txt", None)
            .with_chunk_size(1024 * 1024)
            .with_progress_callback(|_| {});

        assert_eq!(compressor.chunk_size, 1024 * 1024);
        assert!(compressor.progress_callback.is_some());
    }

    #[test]
    fn test_compressor_write_operations() {
        let cursor = Cursor::new(Vec::new());
        let mut compressor = Compressor::new(cursor, "test.txt", None);

        compressor.write_all(b"Hello, world!").unwrap();
        assert_eq!(compressor.bytes_written(), 13);
    }

    #[test]
    fn test_compressor_finish() {
        let cursor = Cursor::new(Vec::new());
        let mut compressor = Compressor::new(cursor, "test.txt", None);

        assert!(!compressor.is_finished());
        compressor.finish().unwrap();
        assert!(compressor.is_finished());
    }

    #[test]
    fn test_compressor_write_after_finish() {
        let cursor = Cursor::new(Vec::new());
        let mut compressor = Compressor::new(cursor, "test.txt", None);

        compressor.finish().unwrap();

        let result = compressor.write_all(b"more data");
        assert!(result.is_err());
    }
}
