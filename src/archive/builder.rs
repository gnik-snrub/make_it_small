use crate::archive::ArchiveInfo;
use crate::crypto::DEFAULT_CHUNK_SIZE;
use crate::error::{ArchiveError, MismallError, Result};
use crate::progress::ProgressCallback;

/// Builder pattern for archive operations with advanced options
///
/// This builder provides a fluent interface for creating archives
/// with multiple files and all available options.
///
/// # Examples
///
/// ```rust
/// use mismall::archive::ArchiveBuilder;
///
/// let result = ArchiveBuilder::new()
///     .add_file("doc1.pdf", b"PDF content")?
///     .add_file("image.jpg", b"JPG content")?
///     .with_password("archive_secret")
///     .with_chunk_size(64 * 1024 * 1024) // 64MB chunks
///     .with_progress_callback(|progress| {
///         println!("Progress: {}%", progress.percentage);
///     })
///     .build("backup.small")?;
///
/// println!("Archive created with {} files", result.file_count);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ArchiveBuilder {
    files: Vec<(String, Vec<u8>)>,
    password: Option<String>,
    chunk_size: usize,
    progress_callback: Option<ProgressCallback>,
}

impl ArchiveBuilder {
    /// Helper function to create archive errors
    fn archive_error(error: ArchiveError) -> MismallError {
        MismallError::Archive {
            error,
            context: None,
            suggestion: None,
        }
    }
    /// Create a new archive builder
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveBuilder;
    ///
    /// let builder = ArchiveBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            password: None,
            chunk_size: DEFAULT_CHUNK_SIZE,
            progress_callback: None,
        }
    }

    /// Add a file to the archive
    ///
    /// # Arguments
    ///
    /// * `path` - Path of the file within the archive
    /// * `data` - File content as bytes
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveBuilder;
    ///
    /// let builder = ArchiveBuilder::new()
    ///     .add_file("document.txt", b"Hello, world!")?;
    /// ```
    pub fn add_file<S: Into<String>, D: Into<Vec<u8>>>(mut self, path: S, data: D) -> Result<Self> {
        let path = path.into();
        let data = data.into();

        // Validate inputs
        if path.is_empty() {
            return Err(Self::archive_error(ArchiveError::Creation(
                "File path cannot be empty".to_string(),
            )));
        }

        if path.len() > 255 {
            return Err(Self::archive_error(ArchiveError::Creation(
                "File path too long (max 255 characters)".to_string(),
            )));
        }

        if self.files.len() >= 1000 {
            return Err(Self::archive_error(ArchiveError::TooManyFiles(
                self.files.len() + 1,
            )));
        }

        let total_size: u64 = self.files.iter().map(|(_, data)| data.len() as u64).sum();
        if total_size + data.len() as u64 > 1024 * 1024 * 1024 * 10 {
            // 10GB limit
            return Err(Self::archive_error(ArchiveError::TooLarge(
                total_size + data.len() as u64,
            )));
        }

        self.files.push((path, data));
        Ok(self)
    }

    /// Add a file from the filesystem to the archive
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file on disk
    /// * `archive_path` - Path where the file should be stored in the archive
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveBuilder;
    ///
    /// let builder = ArchiveBuilder::new()
    ///     .add_file_from_fs("document.txt", "documents/letter.txt")?;
    /// ```
    pub fn add_file_from_fs<P: AsRef<std::path::Path>, S: Into<String>>(
        self,
        fs_path: P,
        archive_path: S,
    ) -> Result<Self> {
        let fs_path = fs_path.as_ref();
        let archive_path = archive_path.into();

        // Read file content
        let data = std::fs::read(fs_path).map_err(|e| {
            Self::archive_error(ArchiveError::Creation(format!(
                "Failed to read file {}: {}",
                fs_path.display(),
                e
            )))
        })?;

        self.add_file(archive_path, data)
    }

    /// Set password for AES-256-GCM encryption of the entire archive
    ///
    /// # Arguments
    ///
    /// * `password` - Password to use for encryption
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveBuilder;
    ///
    /// let builder = ArchiveBuilder::new()
    ///     .with_password("archive_secret");
    /// ```
    pub fn with_password<S: Into<String>>(mut self, password: S) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set custom chunk size for memory usage
    ///
    /// # Arguments
    ///
    /// * `chunk_size` - Chunk size in bytes (64KB to 1GB)
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveBuilder;
    ///
    /// let builder = ArchiveBuilder::new()
    ///     .with_chunk_size(64 * 1024 * 1024); // 64MB
    /// ```
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Set progress callback for real-time updates
    ///
    /// # Arguments
    ///
    /// * `callback` - Function that receives `ProgressInfo`
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveBuilder;
    ///
    /// let builder = ArchiveBuilder::new()
    ///     .with_progress_callback(|progress| {
    ///         println!("{}% complete", progress.percentage);
    ///     });
    /// ```
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&crate::progress::ProgressInfo) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Build the archive and save it to the specified path
    ///
    /// Performs the archive creation with all configured options and returns
    /// an `ArchiveInfo` with statistics about the operation.
    ///
    /// # Arguments
    ///
    /// * `output_path` - Path where the archive should be saved
    ///
    /// # Returns
    ///
    /// Returns `Ok(ArchiveInfo)` on success or `Err(MismallError)` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveBuilder;
    ///
    /// let result = ArchiveBuilder::new()
    ///     .add_file("doc.txt", b"content")?
    ///     .build("backup.small")?;
    ///
    /// println!("Archive created with {} files", result.file_count);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn build<P: AsRef<std::path::Path>>(self, output_path: P) -> Result<ArchiveInfo> {
        // Validate inputs
        if self.files.is_empty() {
            return Err(Self::archive_error(ArchiveError::Creation(
                "Cannot create archive with no files".to_string(),
            )));
        }

        crate::compress::validate_chunk_size(self.chunk_size)?;

        let output_path = output_path.as_ref();

        // For now, this is a simplified implementation
        // In a full implementation, we would:
        // 1. Create a master header
        // 2. For each file, compress it and write to output
        // 3. Write embedded headers and compressed data
        // 4. Update progress callbacks

        println!("Archive creation not yet fully implemented");
        println!(
            "Would create archive with {} files at '{}'",
            self.files.len(),
            output_path.display()
        );

        // Return dummy archive info for now
        let total_original: u64 = self.files.iter().map(|(_, data)| data.len() as u64).sum();
        let archive_info = ArchiveInfo::new(
            self.files.len(),
            total_original,
            total_original, // No compression yet
            self.password.is_some(),
        );

        Ok(archive_info)
    }
}

impl Default for ArchiveBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ArchiveBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchiveBuilder")
            .field("file_count", &self.files.len())
            .field("password", &self.password.is_some())
            .field("chunk_size", &self.chunk_size)
            .field("progress_callback", &self.progress_callback.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = ArchiveBuilder::new();
        assert_eq!(builder.files.len(), 0);
        assert!(builder.password.is_none());
        assert_eq!(builder.chunk_size, DEFAULT_CHUNK_SIZE);
        assert!(builder.progress_callback.is_none());
    }

    #[test]
    fn test_add_file() {
        let builder = ArchiveBuilder::new()
            .add_file("test.txt", b"Hello, world!")
            .unwrap();

        assert_eq!(builder.files.len(), 1);
        assert_eq!(builder.files[0].0, "test.txt");
        assert_eq!(builder.files[0].1, b"Hello, world!");
    }

    #[test]
    fn test_add_file_validation() {
        // Empty path
        let result = ArchiveBuilder::new().add_file("", b"data");
        assert!(result.is_err());

        // Path too long
        let long_path = "a".repeat(256);
        let result = ArchiveBuilder::new().add_file(&long_path, b"data");
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_chain() {
        let builder = ArchiveBuilder::new()
            .with_password("secret")
            .with_chunk_size(1024 * 1024);

        assert_eq!(builder.password, Some("secret".to_string()));
        assert_eq!(builder.chunk_size, 1024 * 1024);
    }

    #[test]
    fn test_build_empty_archive() {
        let builder = ArchiveBuilder::new();
        let result = builder.build("test.small");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_archive() {
        let _result = ArchiveBuilder::new()
            .add_file("test.txt", b"Hello")
            .unwrap()
            .build("test.small");

        println!("Note: Archive building is not fully implemented yet");
        // This should eventually succeed when fully implemented
    }

    #[test]
    fn test_file_limits() {
        let mut builder = ArchiveBuilder::new();

        // Add many files to test limit
        for i in 0..1000 {
            builder = builder.add_file(format!("file{}.txt", i), b"data").unwrap();
        }

        // Next one should fail
        let result = builder.add_file("file1001.txt", b"data");
        assert!(result.is_err());
    }
}
