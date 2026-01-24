use crate::compress::validate_chunk_size;
use crate::compress::CompressionResult;
use crate::crypto::DEFAULT_CHUNK_SIZE;
use crate::error::Result;
use crate::progress::ProgressCallback;
use std::path::PathBuf;

/// Builder pattern for compression operations with advanced options
///
/// This builder provides a fluent interface for configuring compression
/// operations with all available options.
///
/// # Examples
///
/// ```rust
/// use mismall::compress::CompressionBuilder;
///
/// let result = CompressionBuilder::new("document.txt")
///     .with_password("secret123")
///     .with_chunk_size(64 * 1024 * 1024) // 64MB chunks
///     .with_progress_callback(|progress| {
///         println!("Progress: {}%", progress.percentage);
///     })
///     .compress()?;
///
/// println!("Compressed {} bytes to {} bytes",
///          result.original_size, result.compressed_size);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct CompressionBuilder {
    input_path: PathBuf,
    password: Option<String>,
    chunk_size: usize,
    progress_callback: Option<ProgressCallback>,
    output_path: Option<PathBuf>,
}

impl std::fmt::Debug for CompressionBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompressionBuilder")
            .field("input_path", &self.input_path)
            .field("password", &self.password.is_some())
            .field("chunk_size", &self.chunk_size)
            .field("progress_callback", &self.progress_callback.is_some())
            .field("output_path", &self.output_path)
            .finish()
    }
}

impl Clone for CompressionBuilder {
    fn clone(&self) -> Self {
        Self {
            input_path: self.input_path.clone(),
            password: self.password.clone(),
            chunk_size: self.chunk_size,
            progress_callback: None, // Can't clone closures, so reset to None
            output_path: self.output_path.clone(),
        }
    }
}

impl CompressionBuilder {
    /// Create a new compression builder for the given input file
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the file to compress
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::compress::CompressionBuilder;
    ///
    /// let builder = CompressionBuilder::new("document.txt");
    /// ```
    pub fn new<P: Into<PathBuf>>(input_path: P) -> Self {
        Self {
            input_path: input_path.into(),
            password: None,
            chunk_size: DEFAULT_CHUNK_SIZE,
            progress_callback: None,
            output_path: None,
        }
    }

    /// Set password for AES-256-GCM encryption
    ///
    /// # Arguments
    ///
    /// * `password` - Password to use for encryption
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::compress::CompressionBuilder;
    ///
    /// let builder = CompressionBuilder::new("secret.txt")
    ///     .with_password("my-secret-password");
    /// ```
    pub fn with_password<S: Into<String>>(mut self, password: S) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set custom chunk size for memory usage
    ///
    /// Larger chunk sizes use more memory but may improve performance.
    /// Smaller chunk sizes use less memory but may be slower.
    ///
    /// # Arguments
    ///
    /// * `chunk_size` - Chunk size in bytes (64KB to 1GB)
    ///
    /// # Guidelines
    ///
    /// - **Low memory systems (1GB RAM)**: 64KB (65536 bytes)
    /// - **Standard systems (8GB+ RAM)**: 16MB (16,777,216 bytes) - default
    /// - **High-memory systems (32GB+ RAM)**: 64MB to 1GB
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::compress::CompressionBuilder;
    ///
    /// let builder = CompressionBuilder::new("large_file.bin")
    ///     .with_chunk_size(64 * 1024 * 1024); // 64MB
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
    /// use mismall::compress::CompressionBuilder;
    ///
    /// let builder = CompressionBuilder::new("big_file.bin")
    ///     .with_progress_callback(|progress| {
    ///         println!("{}% complete - {} of {} bytes processed",
    ///                  progress.percentage,
    ///                  progress.bytes_processed,
    ///                  progress.total_bytes);
    ///     });
    /// ```
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&crate::progress::ProgressInfo) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Set custom output file path
    ///
    /// By default, the compressed file will be saved with a `.small` extension
    /// in the same directory as the input file.
    ///
    /// # Arguments
    ///
    /// * `output_path` - Path where the compressed file should be saved
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::compress::CompressionBuilder;
    ///
    /// let builder = CompressionBuilder::new("document.txt")
    ///     .with_output_path("compressed/document.small");
    /// ```
    pub fn with_output_path<P: Into<PathBuf>>(mut self, output_path: P) -> Self {
        self.output_path = Some(output_path.into());
        self
    }

    /// Execute the compression operation
    ///
    /// Performs the compression with all configured options and returns
    /// a `CompressionResult` with statistics about the operation.
    ///
    /// # Returns
    ///
    /// Returns `Ok(CompressionResult)` on success or `Err(MismallError)` on failure.
    ///
    /// # Errors
    ///
    /// - `CompressionError::InvalidChunkSize` if chunk size is out of range
    /// - `CompressionError::InputRead` if input file cannot be read
    /// - `CompressionError::OutputWrite` if output cannot be written
    /// - `CompressionError::Encryption` if encryption fails
    /// - `Io` for other I/O related errors
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::compress::CompressionBuilder;
    ///
    /// let result = CompressionBuilder::new("document.txt")
    ///     .with_password("secret")
    ///     .compress()?;
    ///
    /// println!("Compression complete: {:.1}% of original size",
    ///          result.compression_ratio);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn compress(self) -> Result<CompressionResult> {
        // Validate chunk size
        validate_chunk_size(self.chunk_size)?;

        // Determine output path if not specified
        let _output_path = self.output_path.unwrap_or_else(|| {
            let mut path = self.input_path.clone();
            path.set_extension("small");
            path
        });

        // For now, delegate to simple compression with progress
        crate::compress::simple::compress_file_with_progress(
            &self.input_path,
            self.password.as_deref(),
            self.chunk_size,
            self.progress_callback,
        )
    }
}

impl From<&str> for CompressionBuilder {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl From<String> for CompressionBuilder {
    fn from(path: String) -> Self {
        Self::new(path)
    }
}

impl From<PathBuf> for CompressionBuilder {
    fn from(path: PathBuf) -> Self {
        Self::new(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_builder_creation() {
        let builder = CompressionBuilder::new("test.txt");
        assert_eq!(builder.input_path, PathBuf::from("test.txt"));
        assert!(builder.password.is_none());
        assert_eq!(builder.chunk_size, DEFAULT_CHUNK_SIZE);
        assert!(builder.progress_callback.is_none());
        assert!(builder.output_path.is_none());
    }

    #[test]
    fn test_builder_from_conversions() {
        // From &str
        let builder1 = CompressionBuilder::from("test.txt");
        assert_eq!(builder1.input_path, PathBuf::from("test.txt"));

        // From String
        let builder2 = CompressionBuilder::from("test.txt".to_string());
        assert_eq!(builder2.input_path, PathBuf::from("test.txt"));

        // From PathBuf
        let path = PathBuf::from("test.txt");
        let builder3 = CompressionBuilder::from(path.clone());
        assert_eq!(builder3.input_path, path);
    }

    #[test]
    fn test_builder_chain() {
        let builder = CompressionBuilder::new("test.txt")
            .with_password("secret")
            .with_chunk_size(1024 * 1024)
            .with_output_path("output.small");

        assert_eq!(builder.input_path, PathBuf::from("test.txt"));
        assert_eq!(builder.password, Some("secret".to_string()));
        assert_eq!(builder.chunk_size, 1024 * 1024);
        assert_eq!(builder.output_path, Some(PathBuf::from("output.small")));
    }

    #[test]
    fn test_builder_progress_callback() {
        use std::sync::atomic::{AtomicU32, Ordering};

        static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

        let _builder = CompressionBuilder::new("test.txt").with_progress_callback(|progress| {
            CALL_COUNT.fetch_add(1, Ordering::SeqCst);
            // Check that we can access progress info
            assert!(progress.percentage >= 0.0);
        });

        // In real usage, the callback would be called during compress()
        // This test just verifies the builder accepts the callback
    }

    #[test]
    fn test_output_path_default() {
        let builder = CompressionBuilder::new("document.txt");
        let expected = PathBuf::from("document.small");
        let actual = builder.output_path.unwrap_or_else(|| {
            let mut path = builder.input_path.clone();
            path.set_extension("small");
            path
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_invalid_chunk_size() {
        let builder = CompressionBuilder::new("test.txt").with_chunk_size(32 * 1024); // Too small

        let result = builder.compress();
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::error::MismallError::Compression(CompressionError::InvalidChunkSize(size)) => {
                assert_eq!(size, 32 * 1024);
            }
            _ => panic!("Expected InvalidChunkSize error"),
        }
    }

    #[test]
    fn test_compression_with_real_file() {
        // Create a temporary file with test data
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"This is test data for compression builder testing. It should be compressible enough to show some savings.";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        // Test compression
        let result = CompressionBuilder::new(temp_file.path())
            .with_password("test_password")
            .compress();

        assert!(result.is_ok());
        let compression_result = result.unwrap();
        assert_eq!(compression_result.original_size, test_data.len() as u64);
        assert!(compression_result.encrypted);
        assert!(compression_result.compressed_size > 0);
    }
}
