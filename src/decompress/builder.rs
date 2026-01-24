use crate::crypto::DEFAULT_CHUNK_SIZE;
use crate::decompress::DecompressionResult;
use crate::error::Result;
use crate::progress::ProgressCallback;
use std::path::PathBuf;

/// Builder pattern for decompression operations with advanced options
///
/// This builder provides a fluent interface for configuring decompression
/// operations with all available options.
///
/// # Examples
///
/// ```rust
/// use mismall::decompress::DecompressionBuilder;
///
/// let result = DecompressionBuilder::new("document.txt.small")
///     .with_password("secret123")
///     .with_chunk_size(64 * 1024 * 1024) // 64MB chunks
///     .with_progress_callback(|progress| {
///         println!("Progress: {}%", progress.percentage);
///     })
///     .decompress()?;
///
/// println!("Decompressed {} bytes as {}",
///          result.original_size, result.original_filename);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct DecompressionBuilder {
    input_path: PathBuf,
    password: Option<String>,
    chunk_size: usize,
    progress_callback: Option<ProgressCallback>,
    output_path: Option<PathBuf>,
}

impl DecompressionBuilder {
    /// Create a new decompression builder for the given input file
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the .small file to decompress
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::decompress::DecompressionBuilder;
    ///
    /// let builder = DecompressionBuilder::new("document.txt.small");
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

    /// Set password for AES-256-GCM decryption
    ///
    /// # Arguments
    ///
    /// * `password` - Password to use for decryption
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::decompress::DecompressionBuilder;
    ///
    /// let builder = DecompressionBuilder::new("encrypted.small")
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
    /// use mismall::decompress::DecompressionBuilder;
    ///
    /// let builder = DecompressionBuilder::new("large.small")
    ///     .with_chunk_size(64 * 1024 * 1024); // 64MB
    /// ```
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Set progress callback for real-time updates
    ///
    /// The callback will be called periodically with progress information
    /// during the decompression process.
    ///
    /// # Arguments
    ///
    /// * `callback` - Function that receives `ProgressInfo`
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::decompress::DecompressionBuilder;
    ///
    /// let builder = DecompressionBuilder::new("big_file.small")
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
    /// By default, the decompressed file will be saved with the original filename
    /// in the same directory as the input file.
    ///
    /// # Arguments
    ///
    /// * `output_path` - Path where the decompressed file should be saved
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::decompress::DecompressionBuilder;
    ///
    /// let builder = DecompressionBuilder::new("document.txt.small")
    ///     .with_output_path("decompressed/document_restored.txt");
    /// ```
    pub fn with_output_path<P: Into<PathBuf>>(mut self, output_path: P) -> Self {
        self.output_path = Some(output_path.into());
        self
    }

    /// Execute the decompression operation
    ///
    /// Performs the decompression with all configured options and returns
    /// a `DecompressionResult` with statistics about the operation.
    ///
    /// # Returns
    ///
    /// Returns `Ok(DecompressionResult)` on success or `Err(MismallError)` on failure.
    ///
    /// # Errors
    ///
    /// - `DecompressionError::PasswordRequired` if file is encrypted but no password provided
    /// - `DecompressionError::InvalidPassword` if password is incorrect
    /// - `DecompressionError::InvalidFormat` if file format is invalid
    /// - `DecompressionError::CorruptedData` if data is corrupted
    /// - `DecompressionError::ChecksumMismatch` if integrity check fails
    /// - `Io` for other I/O related errors
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::decompress::DecompressionBuilder;
    ///
    /// let result = DecompressionBuilder::new("document.txt.small")
    ///     .with_password("secret")
    ///     .decompress()?;
    ///
    /// println!("Decompression complete: {} bytes", result.original_size);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn decompress(self) -> Result<DecompressionResult> {
        // Validate chunk size
        crate::compress::validate_chunk_size(self.chunk_size)?;

        // For now, delegate to simple decompression with progress
        crate::decompress::simple::decompress_file_with_progress(
            &self.input_path,
            self.password.as_deref(),
            self.chunk_size,
            self.progress_callback,
        )
    }
}

impl std::fmt::Debug for DecompressionBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecompressionBuilder")
            .field("input_path", &self.input_path)
            .field("password", &self.password.is_some())
            .field("chunk_size", &self.chunk_size)
            .field("progress_callback", &self.progress_callback.is_some())
            .field("output_path", &self.output_path)
            .finish()
    }
}

impl From<&str> for DecompressionBuilder {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl From<String> for DecompressionBuilder {
    fn from(path: String) -> Self {
        Self::new(path)
    }
}

impl From<PathBuf> for DecompressionBuilder {
    fn from(path: PathBuf) -> Self {
        Self::new(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = DecompressionBuilder::new("test.small");
        assert_eq!(builder.input_path, PathBuf::from("test.small"));
        assert!(builder.password.is_none());
        assert_eq!(builder.chunk_size, DEFAULT_CHUNK_SIZE);
        assert!(builder.progress_callback.is_none());
        assert!(builder.output_path.is_none());
    }

    #[test]
    fn test_builder_from_conversions() {
        // From &str
        let builder1 = DecompressionBuilder::from("test.small");
        assert_eq!(builder1.input_path, PathBuf::from("test.small"));

        // From String
        let builder2 = DecompressionBuilder::from("test.small".to_string());
        assert_eq!(builder2.input_path, PathBuf::from("test.small"));

        // From PathBuf
        let path = PathBuf::from("test.small");
        let builder3 = DecompressionBuilder::from(path.clone());
        assert_eq!(builder3.input_path, path);
    }

    #[test]
    fn test_builder_chain() {
        let builder = DecompressionBuilder::new("test.small")
            .with_password("secret")
            .with_chunk_size(1024 * 1024)
            .with_output_path("output.txt");

        assert_eq!(builder.input_path, PathBuf::from("test.small"));
        assert_eq!(builder.password, Some("secret".to_string()));
        assert_eq!(builder.chunk_size, 1024 * 1024);
        assert_eq!(builder.output_path, Some(PathBuf::from("output.txt")));
    }

    #[test]
    fn test_invalid_chunk_size() {
        let builder = DecompressionBuilder::new("test.small").with_chunk_size(32 * 1024); // Too small

        let result = builder.decompress();
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::error::MismallError::Compression(
                crate::error::CompressionError::InvalidChunkSize(size),
            ) => {
                assert_eq!(size, 32 * 1024);
            }
            _ => panic!("Expected InvalidChunkSize error"),
        }
    }
}
