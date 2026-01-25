//! Error types for mismall compression library
//!
//! This module defines a comprehensive error hierarchy that provides detailed
//! information about what went wrong during compression, decompression,
//! or archive operations, including rich context and helpful suggestions.
//!
//! # Examples
//!
//! ```rust
//! use mismall::error::{MismallError, Result};
//!
//! fn process_file() -> Result<()> {
//!     // This will automatically convert std::io::Error to MismallError
//!     std::fs::read_to_string("file.txt")?;
//!     Ok(())
//! }
//!
//! fn handle_error(error: MismallError) {
//!     match error {
//!         MismallError::Compression { error, .. } => {
//!             eprintln!("Compression failed: {}", error);
//!         }
//!         MismallError::Decompression { error, .. } => {
//!             eprintln!("Decompression failed: {}", error);
//!         }
//!         MismallError::Archive { error, .. } => {
//!             eprintln!("Archive operation failed: {}", error);
//!         }
//!         MismallError::Io { error, .. } => {
//!             eprintln!("I/O error: {}", error);
//!         }
//!         MismallError::InvalidInput { message, .. } => {
//!             eprintln!("Invalid input: {}", message);
//!         }
//!         MismallError::Unsupported { operation, .. } => {
//!             eprintln!("Unsupported operation: {}", operation);
//!         }
//!     }
//! }
//! ```

pub mod context;

use context::{ErrorContext, Suggestion};
use std::fmt;

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, MismallError>;

/// Specific compression error types
#[derive(Debug)]
pub enum CompressionError {
    /// Invalid chunk size provided
    InvalidChunkSize(usize),
    /// Failed to read input data
    InputRead(String),
    /// Failed to write output data
    OutputWrite(String),
    /// Compression algorithm failed
    CompressionFailed(String),
    /// Encryption error during compression
    Encryption(String),
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionError::InvalidChunkSize(size) => {
                write!(
                    f,
                    "Invalid chunk size: {} (must be between 1KB and 100MB)",
                    size
                )
            }
            CompressionError::InputRead(msg) => {
                write!(f, "Failed to read input: {}", msg)
            }
            CompressionError::OutputWrite(msg) => {
                write!(f, "Failed to write output: {}", msg)
            }
            CompressionError::CompressionFailed(msg) => {
                write!(f, "Compression failed: {}", msg)
            }
            CompressionError::Encryption(msg) => {
                write!(f, "Encryption error: {}", msg)
            }
        }
    }
}

impl std::error::Error for CompressionError {}

/// Specific decompression error types
#[derive(Debug)]
pub enum DecompressionError {
    /// Invalid chunk size provided
    InvalidChunkSize(usize),
    /// Failed to read input data
    InputRead(String),
    /// Failed to write output data
    OutputWrite(String),
    /// Decompression algorithm failed
    DecompressionFailed(String),
    /// Invalid file format
    InvalidFormat(String),
    /// Corrupted data detected
    CorruptedData(String),
    /// Decryption error during decompression
    DecryptionError(String),
}

impl fmt::Display for DecompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecompressionError::InvalidChunkSize(size) => {
                write!(
                    f,
                    "Invalid chunk size: {} (must be between 1KB and 100MB)",
                    size
                )
            }
            DecompressionError::InputRead(msg) => {
                write!(f, "Failed to read input: {}", msg)
            }
            DecompressionError::OutputWrite(msg) => {
                write!(f, "Failed to write output: {}", msg)
            }
            DecompressionError::DecompressionFailed(msg) => {
                write!(f, "Decompression failed: {}", msg)
            }
            DecompressionError::InvalidFormat(msg) => {
                write!(f, "Invalid file format: {}", msg)
            }
            DecompressionError::CorruptedData(msg) => {
                write!(f, "Corrupted data: {}", msg)
            }
            DecompressionError::DecryptionError(msg) => {
                write!(f, "Decryption error: {}", msg)
            }
        }
    }
}

impl std::error::Error for DecompressionError {}

/// Specific archive error types
#[cfg(feature = "archives")]
#[derive(Debug)]
pub enum ArchiveError {
    /// Failed to create archive
    Creation(String),
    /// Too many files in archive
    TooManyFiles(usize),
    /// Archive too large
    TooLarge(u64),
    /// Failed to extract from archive
    Extraction(String),
    /// Invalid archive format
    InvalidFormat(String),
    /// File not found in archive
    FileNotFound(String),
}

#[cfg(feature = "archives")]
impl fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchiveError::Creation(msg) => {
                write!(f, "Archive creation failed: {}", msg)
            }
            ArchiveError::TooManyFiles(count) => {
                write!(f, "Too many files in archive: {} (max 65535)", count)
            }
            ArchiveError::TooLarge(size) => {
                write!(f, "Archive too large: {} bytes (max 4GB)", size)
            }
            ArchiveError::Extraction(msg) => {
                write!(f, "Archive extraction failed: {}", msg)
            }
            ArchiveError::InvalidFormat(msg) => {
                write!(f, "Invalid archive format: {}", msg)
            }
            ArchiveError::FileNotFound(name) => {
                write!(f, "File not found in archive: {}", name)
            }
        }
    }
}

#[cfg(feature = "archives")]
impl std::error::Error for ArchiveError {}

/// Main error type for all mismall operations
#[derive(Debug)]
#[allow(clippy::result_large_err)] // All variants are large due to context/suggestions, this is expected
#[allow(clippy::result_large_err)]
pub enum MismallError {
    /// Errors that occur during compression operations
    Compression {
        error: CompressionError,
        context: Option<ErrorContext>,
        suggestion: Option<Suggestion>,
    },
    /// Errors that occur during decompression operations
    Decompression {
        error: DecompressionError,
        context: Option<ErrorContext>,
        suggestion: Option<Suggestion>,
    },
    /// Errors that occur during archive operations
    #[cfg(feature = "archives")]
    Archive {
        error: ArchiveError,
        context: Option<ErrorContext>,
        suggestion: Option<Suggestion>,
    },
    /// I/O related errors
    Io {
        error: std::io::Error,
        context: Option<ErrorContext>,
        suggestion: Option<Suggestion>,
    },
    /// Invalid input parameters
    InvalidInput {
        message: String,
        context: Option<ErrorContext>,
        suggestion: Option<Suggestion>,
    },
    /// Unsupported operation
    Unsupported {
        operation: String,
        context: Option<ErrorContext>,
        suggestion: Option<Suggestion>,
    },
}

impl fmt::Display for MismallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MismallError::Compression { error, .. } => {
                write!(f, "Compression error: {}", error)
            }
            MismallError::Decompression { error, .. } => {
                write!(f, "Decompression error: {}", error)
            }
            #[cfg(feature = "archives")]
            MismallError::Archive { error, .. } => {
                write!(f, "Archive error: {}", error)
            }
            MismallError::Io { error, .. } => {
                write!(f, "I/O error: {}", error)
            }
            MismallError::InvalidInput { message, .. } => {
                write!(f, "Invalid input: {}", message)
            }
            MismallError::Unsupported { operation, .. } => {
                write!(f, "Unsupported operation: {}", operation)
            }
        }
    }
}

impl std::error::Error for MismallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MismallError::Compression { error, .. } => Some(error),
            MismallError::Decompression { error, .. } => Some(error),
            #[cfg(feature = "archives")]
            MismallError::Archive { error, .. } => Some(error),
            MismallError::Io { error, .. } => Some(error),
            MismallError::InvalidInput { .. } | MismallError::Unsupported { .. } => None,
        }
    }
}

/// Extension trait for adding context to MismallError results
pub trait ErrorExt {
    /// Add context information to an error
    fn with_context(self, context: ErrorContext) -> Self;

    /// Add a helpful suggestion to an error
    fn with_suggestion(self, suggestion: Suggestion) -> Self;
}

impl ErrorExt for crate::error::Result<()> {
    fn with_context(self, context: ErrorContext) -> Self {
        self.map_err(|e| match e {
            MismallError::Compression { error, .. } => MismallError::Compression {
                error,
                context: Some(context),
                suggestion: None,
            },
            MismallError::Decompression { error, .. } => MismallError::Decompression {
                error,
                context: Some(context),
                suggestion: None,
            },
            #[cfg(feature = "archives")]
            MismallError::Archive { error, .. } => MismallError::Archive {
                error,
                context: Some(context),
                suggestion: None,
            },
            MismallError::Io { error, .. } => MismallError::Io {
                error,
                context: Some(context),
                suggestion: None,
            },
            MismallError::InvalidInput { message, .. } => MismallError::InvalidInput {
                message,
                context: Some(context),
                suggestion: None,
            },
            MismallError::Unsupported { operation, .. } => MismallError::Unsupported {
                operation,
                context: Some(context),
                suggestion: None,
            },
        })
    }

    fn with_suggestion(self, suggestion: Suggestion) -> Self {
        self.map_err(|e| match e {
            MismallError::Compression { error, .. } => MismallError::Compression {
                error,
                context: None,
                suggestion: Some(suggestion),
            },
            MismallError::Decompression { error, .. } => MismallError::Decompression {
                error,
                context: None,
                suggestion: Some(suggestion),
            },
            #[cfg(feature = "archives")]
            MismallError::Archive { error, .. } => MismallError::Archive {
                error,
                context: None,
                suggestion: Some(suggestion),
            },
            MismallError::Io { error, .. } => MismallError::Io {
                error,
                context: None,
                suggestion: Some(suggestion),
            },
            MismallError::InvalidInput { message, .. } => MismallError::InvalidInput {
                message,
                context: None,
                suggestion: Some(suggestion),
            },
            MismallError::Unsupported { operation, .. } => MismallError::Unsupported {
                operation,
                context: None,
                suggestion: Some(suggestion),
            },
        })
    }
}

/// Enhanced error with rich context and suggestions
#[derive(Debug)]
pub struct EnhancedError<T> {
    /// The original error that occurred
    pub original: T,

    /// Context information about when and where an error occurred
    pub context: Option<ErrorContext>,

    /// Helpful suggestion for resolving the error
    pub suggestion: Option<Suggestion>,
}

impl<T> EnhancedError<T> {
    /// Create a new enhanced error
    pub fn new(original: T) -> Self {
        Self {
            original,
            context: None,
            suggestion: None,
        }
    }

    /// Create a new enhanced error with context
    pub fn new_with_context(original: T, context: ErrorContext) -> Self {
        Self {
            original,
            context: Some(context),
            suggestion: None,
        }
    }

    /// Create a new enhanced error with suggestion
    pub fn new_with_suggestion(original: T, suggestion: Suggestion) -> Self {
        Self {
            original,
            context: None,
            suggestion: Some(suggestion),
        }
    }
}

// Conversion traits for common error types
impl From<std::io::Error> for MismallError {
    fn from(err: std::io::Error) -> Self {
        MismallError::Io {
            error: err,
            context: None,
            suggestion: None,
        }
    }
}

impl From<CompressionError> for MismallError {
    fn from(err: CompressionError) -> Self {
        MismallError::Compression {
            error: err,
            context: None,
            suggestion: None,
        }
    }
}

impl From<DecompressionError> for MismallError {
    fn from(err: DecompressionError) -> Self {
        MismallError::Decompression {
            error: err,
            context: None,
            suggestion: None,
        }
    }
}

#[cfg(feature = "archives")]
impl From<ArchiveError> for MismallError {
    fn from(err: ArchiveError) -> Self {
        MismallError::Archive {
            error: err,
            context: None,
            suggestion: None,
        }
    }
}

impl From<Box<dyn std::error::Error>> for MismallError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        MismallError::InvalidInput {
            message: err.to_string(),
            context: None,
            suggestion: None,
        }
    }
}

impl From<&str> for MismallError {
    fn from(err: &str) -> Self {
        MismallError::InvalidInput {
            message: err.to_string(),
            context: None,
            suggestion: None,
        }
    }
}

impl From<String> for MismallError {
    fn from(err: String) -> Self {
        MismallError::InvalidInput {
            message: err,
            context: None,
            suggestion: None,
        }
    }
}
