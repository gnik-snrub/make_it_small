use crate::archive::{ArchiveInfo, FileInfo};
use crate::crypto::DEFAULT_CHUNK_SIZE;
use crate::error::{ArchiveError, Result};
use crate::progress::{ProcessingStage, ProgressCallback};
use std::path::PathBuf;

/// Builder pattern for archive extraction operations
///
/// This builder provides a fluent interface for extracting files
/// from archives with all available options.
///
/// # Examples
///
/// ```rust
/// use mismall::archive::ArchiveExtractor;
///
/// let result = ArchiveExtractor::new("backup.small")
///     .with_password("archive_secret")
///     .with_output_dir("extracted/")
///     .extract_all()?;
///
/// println!("Extracted {} files", result.file_count);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ArchiveExtractor {
    archive_path: PathBuf,
    password: Option<String>,
    chunk_size: usize,
    progress_callback: Option<ProgressCallback>,
    output_dir: Option<PathBuf>,
}

impl ArchiveExtractor {
    /// Create a new archive extractor for the given archive file
    ///
    /// # Arguments
    ///
    /// * `archive_path` - Path to the .small archive file
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveExtractor;
    ///
    /// let extractor = ArchiveExtractor::new("backup.small");
    /// ```
    pub fn new<P: Into<PathBuf>>(archive_path: P) -> Self {
        Self {
            archive_path: archive_path.into(),
            password: None,
            chunk_size: DEFAULT_CHUNK_SIZE,
            progress_callback: None,
            output_dir: None,
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
    /// use mismall::archive::ArchiveExtractor;
    ///
    /// let extractor = ArchiveExtractor::new("backup.small")
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
    /// use mismall::archive::ArchiveExtractor;
    ///
    /// let extractor = ArchiveExtractor::new("backup.small")
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
    /// use mismall::archive::ArchiveExtractor;
    ///
    /// let extractor = ArchiveExtractor::new("backup.small")
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

    /// Set output directory for extraction
    ///
    /// By default, files will be extracted to the current directory.
    ///
    /// # Arguments
    ///
    /// * `output_dir` - Directory where files should be extracted
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveExtractor;
    ///
    /// let extractor = ArchiveExtractor::new("backup.small")
    ///     .with_output_dir("extracted/");
    /// ```
    pub fn with_output_dir<P: Into<PathBuf>>(mut self, output_dir: P) -> Self {
        self.output_dir = Some(output_dir.into());
        self
    }

    /// Extract all files from the archive
    ///
    /// Performs the extraction with all configured options and returns
    /// an `ArchiveInfo` with statistics about the operation.
    ///
    /// # Returns
    ///
    /// Returns `Ok(ArchiveInfo)` on success or `Err(MismallError)` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveExtractor;
    ///
    /// let result = ArchiveExtractor::new("backup.small")
    ///     .extract_all()?;
    ///
    /// println!("Extracted {} files", result.file_count);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn extract_all(self) -> Result<ArchiveInfo> {
        // Validate chunk size
        crate::compress::validate_chunk_size(self.chunk_size)?;

        let output_dir = self.output_dir.unwrap_or_else(|| {
            std::env::current_dir().map_err(|e| {
                crate::error::MismallError::Archive(ArchiveError::Extraction(format!(
                    "Failed to get current directory: {}",
                    e
                )))
            })?
        });

        crate::archive::simple::extract_archive(
            &self.archive_path,
            &output_dir,
            self.password.as_deref(),
        )
    }

    /// Extract a specific file from the archive
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path of the file within the archive to extract
    /// * `output_path` - Where the extracted file should be saved
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or `Err(MismallError)` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveExtractor;
    ///
    /// ArchiveExtractor::new("backup.small")
    ///     .extract_file("documents/contract.pdf", "restored_contract.pdf")?;
    ///
    /// println!("File extracted successfully");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn extract_file<P: AsRef<std::path::Path>>(
        self,
        file_path: &str,
        output_path: P,
    ) -> Result<()> {
        // Validate chunk size
        crate::compress::validate_chunk_size(self.chunk_size)?;

        crate::archive::simple::extract_file(
            &self.archive_path,
            file_path,
            output_path,
            self.password.as_deref(),
        )
    }

    /// List files in the archive without extracting them
    ///
    /// # Returns
    ///
    /// Returns `Ok((ArchiveInfo, Vec<FileInfo>))` with archive information
    /// and list of files.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::archive::ArchiveExtractor;
    ///
    /// let (info, files) = ArchiveExtractor::new("backup.small")
    ///     .list_contents()?;
    ///
    /// println!("Archive contains {} files:", info.file_count);
    /// for (i, file) in files.iter().enumerate() {
    ///     println!("{}: {} ({} bytes)", i + 1, file.path, file.original_size);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn list_contents(self) -> Result<(ArchiveInfo, Vec<FileInfo>)> {
        crate::archive::simple::list_archive_contents(&self.archive_path)
    }
}

impl From<&str> for ArchiveExtractor {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl From<String> for ArchiveExtractor {
    fn from(path: String) -> Self {
        Self::new(path)
    }
}

impl From<PathBuf> for ArchiveExtractor {
    fn from(path: PathBuf) -> Self {
        Self::new(path)
    }
}

impl std::fmt::Debug for ArchiveExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchiveExtractor")
            .field("archive_path", &self.archive_path)
            .field("password", &self.password.is_some())
            .field("chunk_size", &self.chunk_size)
            .field("progress_callback", &self.progress_callback.is_some())
            .field("output_dir", &self.output_dir)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_creation() {
        let extractor = ArchiveExtractor::new("test.small");
        assert_eq!(extractor.archive_path, PathBuf::from("test.small"));
        assert!(extractor.password.is_none());
        assert_eq!(extractor.chunk_size, DEFAULT_CHUNK_SIZE);
        assert!(extractor.progress_callback.is_none());
        assert!(extractor.output_dir.is_none());
    }

    #[test]
    fn test_extractor_from_conversions() {
        // From &str
        let extractor1 = ArchiveExtractor::from("test.small");
        assert_eq!(extractor1.archive_path, PathBuf::from("test.small"));

        // From String
        let extractor2 = ArchiveExtractor::from("test.small".to_string());
        assert_eq!(extractor2.archive_path, PathBuf::from("test.small"));

        // From PathBuf
        let path = PathBuf::from("test.small");
        let extractor3 = ArchiveExtractor::from(path.clone());
        assert_eq!(extractor3.archive_path, path);
    }

    #[test]
    fn test_extractor_chain() {
        let extractor = ArchiveExtractor::new("test.small")
            .with_password("secret")
            .with_chunk_size(1024 * 1024)
            .with_output_dir("extracted/");

        assert_eq!(extractor.archive_path, PathBuf::from("test.small"));
        assert_eq!(extractor.password, Some("secret".to_string()));
        assert_eq!(extractor.chunk_size, 1024 * 1024);
        assert_eq!(extractor.output_dir, Some(PathBuf::from("extracted/")));
    }

    #[test]
    fn test_list_contents() {
        let extractor = ArchiveExtractor::new("nonexistent.small");
        let result = extractor.list_contents();
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_all_nonexistent() {
        let extractor = ArchiveExtractor::new("nonexistent.small");
        let result = extractor.extract_all();
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_file_nonexistent() {
        let extractor = ArchiveExtractor::new("nonexistent.small");
        let result = extractor.extract_file("test.txt", "output.txt");
        assert!(result.is_err());
    }
}
