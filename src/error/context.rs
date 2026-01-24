//! Error context and suggestion utilities for enhanced error handling
//!
//! This module provides structures to add rich context and helpful suggestions
//! to error messages, making debugging and troubleshooting much easier.

use std::path::PathBuf;
use std::time::SystemTime;

/// Detailed context information about when and where an error occurred
///
/// Provides rich context for error reporting including operation details,
/// file information, and timing information.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The operation being performed when error occurred
    pub operation: String,

    /// Path of the file being processed (if applicable)
    pub file_path: Option<PathBuf>,

    /// Position in the file or stream where error occurred
    pub position: Option<u64>,

    /// Chunk size or memory configuration being used
    pub chunk_size: Option<usize>,

    /// When the error occurred (for debugging and logging)
    pub timestamp: SystemTime,

    /// Additional custom context information
    pub custom_context: Option<String>,
}

impl ErrorContext {
    /// Create new error context for an operation
    ///
    /// # Arguments
    ///
    /// * `operation` - Name of the operation being performed
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::ErrorContext;
    ///
    /// let context = ErrorContext::new("compress_file")
    ///     .with_file_path("data.txt")
    ///     .with_position(1024);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            file_path: None,
            position: None,
            chunk_size: None,
            timestamp: SystemTime::now(),
            custom_context: None,
        }
    }

    /// Add file path to the error context
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file being processed
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::ErrorContext;
    ///
    /// let context = ErrorContext::new("compress_file")
    ///     .with_file_path("/path/to/file.txt");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_file_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Add position information to the error context
    ///
    /// # Arguments
    ///
    /// * `position` - Byte position where error occurred
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::ErrorContext;
    ///
    /// let context = ErrorContext::new("decompress_file")
    ///     .with_position(4096);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_position(mut self, position: u64) -> Self {
        self.position = Some(position);
        self
    }

    /// Add chunk size information to the error context
    ///
    /// # Arguments
    ///
    /// * `chunk_size` - Chunk size being used when error occurred
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::ErrorContext;
    ///
    /// let context = ErrorContext::new("compress_file")
    ///     .with_chunk_size(1024 * 1024);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = Some(chunk_size);
        self
    }

    /// Add custom context information to the error context
    ///
    /// # Arguments
    ///
    /// * `context` - Additional context information
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::ErrorContext;
    ///
    /// let context = ErrorContext::new("decrypt_file")
    ///     .with_custom_context("Using AES-256-GCM encryption");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_custom_context<S: Into<String>>(mut self, context: S) -> Self {
        self.custom_context = Some(context.into());
        self
    }
}

/// Helpful suggestion for resolving an error
///
/// Provides actionable advice to users when errors occur,
/// making troubleshooting much easier.
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Recommended action to resolve the error
    pub action: String,

    /// Reason why this suggestion might help
    pub reason: String,

    /// Error code reference (if applicable)
    pub code: Option<String>,

    /// Link to relevant documentation
    pub documentation_url: Option<String>,
}

impl Suggestion {
    /// Create a new suggestion
    ///
    /// # Arguments
    ///
    /// * `action` - Recommended action to take
    /// * `reason` - Why this action might help
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::Suggestion;
    ///
    /// let suggestion = Suggestion::new(
    ///     "Try increasing chunk size",
    ///     "Current chunk size may be too small for your file"
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(action: &str, reason: &str) -> Self {
        Self {
            action: action.to_string(),
            reason: reason.to_string(),
            code: None,
            documentation_url: None,
        }
    }

    /// Add error code to the suggestion
    ///
    /// # Arguments
    ///
    /// * `code` - Error code reference
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::Suggestion;
    ///
    /// let suggestion = Suggestion::new(
    ///     "Check file permissions",
    ///     "Directory may not be writable"
    /// ).with_code("ERR_PERMISSIONS");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_code<S: Into<String>>(mut self, code: S) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add documentation URL to the suggestion
    ///
    /// # Arguments
    ///
    /// * `url` - URL to relevant documentation
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::Suggestion;
    ///
    /// let suggestion = Suggestion::new(
    ///     "Consult encryption guide",
    ///     "For detailed encryption troubleshooting"
    /// ).with_documentation("https://docs.mismall.com/encryption");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_documentation<S: Into<String>>(mut self, url: S) -> Self {
        self.documentation_url = Some(url.into());
        self
    }
}

/// Extension trait for adding context to existing errors
///
/// This trait allows any error type to be enhanced with rich context
/// and suggestions, making error handling more consistent across the library.
pub trait ErrorExt<T> {
    /// Add context information to the error
    ///
    /// # Arguments
    ///
    /// * `context` - Context information about when/where error occurred
    ///
    /// # Returns
    ///
    /// Enhanced error with context
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::ErrorContext;
    ///
    /// let error = std::io::Error::new(...);
    /// let enhanced_error = error.with_context(ErrorContext::new("read_file"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn with_context(self, context: ErrorContext) -> EnhancedError<T>;

    /// Add a helpful suggestion to the error
    ///
    /// # Arguments
    ///
    /// * `suggestion` - Suggestion for resolving the error
    ///
    /// # Returns
    ///
    /// Enhanced error with suggestion
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::{ErrorContext, Suggestion};
    ///
    /// let error = std::io::Error::new(...);
    /// let suggestion = Suggestion::new("Try again", "Temporary network issue");
    /// let enhanced_error = error.with_suggestion(suggestion);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn with_suggestion(self, suggestion: Suggestion) -> EnhancedError<T>;
}

/// Enhanced error with rich context and suggestions
///
/// This wrapper provides additional debugging information and helpful
/// suggestions for resolving errors.
#[derive(Debug)]
pub struct EnhancedError<T> {
    /// The original error that occurred
    pub original: T,

    /// Context information about when and where the error occurred
    pub context: Option<ErrorContext>,

    /// Helpful suggestion for resolving the error
    pub suggestion: Option<Suggestion>,
}

impl<T> EnhancedError<T> {
    /// Create a new enhanced error
    ///
    /// # Arguments
    ///
    /// * `original` - The original error that occurred
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::EnhancedError;
    ///
    /// let io_error = std::io::Error::new(...);
    /// let enhanced = EnhancedError::new(io_error);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(original: T) -> Self {
        Self {
            original,
            context: None,
            suggestion: None,
        }
    }

    /// Create a new enhanced error with context
    ///
    /// # Arguments
    ///
    /// * `original` - The original error that occurred
    /// * `context` - Context information about the error
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::{EnhancedError, ErrorContext};
    ///
    /// let io_error = std::io::Error::new(...);
    /// let context = ErrorContext::new("compress_file")
    ///     .with_file_path("data.txt");
    /// let enhanced = EnhancedError::new_with_context(io_error, context);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_with_context(original: T, context: ErrorContext) -> Self {
        Self {
            original,
            context: Some(context),
            suggestion: None,
        }
    }

    /// Create a new enhanced error with suggestion
    ///
    /// # Arguments
    ///
    /// * `original` - The original error that occurred
    /// * `suggestion` - Suggestion for resolving the error
    ///
    /// # Example
    ///
    /// ```rust
    /// use mismall::error::context::{EnhancedError, ErrorContext, Suggestion};
    ///
    /// let io_error = std::io::Error::new(...);
    /// let suggestion = Suggestion::new("Try again", "Temporary network issue");
    /// let enhanced = EnhancedError::new_with_suggestion(io_error, suggestion);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_with_suggestion(original: T, suggestion: Suggestion) -> Self {
        Self {
            original,
            context: None,
            suggestion: Some(suggestion),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_builder() {
        let context = ErrorContext::new("test_operation")
            .with_file_path("/test/file.txt")
            .with_position(1024)
            .with_chunk_size(4096);

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.file_path, Some(PathBuf::from("/test/file.txt")));
        assert_eq!(context.position, Some(1024));
        assert_eq!(context.chunk_size, Some(4096));
    }

    #[test]
    fn test_suggestion_builder() {
        let suggestion = Suggestion::new("Check permissions", "Directory not writable")
            .with_code("ERR_PERMISSIONS")
            .with_documentation("https://docs.example.com/permissions");

        assert_eq!(suggestion.action, "Check permissions");
        assert_eq!(suggestion.reason, "Directory not writable");
        assert_eq!(suggestion.code, Some("ERR_PERMISSIONS".to_string()));
        assert_eq!(
            suggestion.documentation_url,
            Some("https://docs.example.com/permissions".to_string())
        );
    }

    #[test]
    fn test_enhanced_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let context = ErrorContext::new("read_file").with_position(100);
        let enhanced = EnhancedError::new_with_context(io_error, context);

        assert!(enhanced.context.is_some());
        assert_eq!(enhanced.context.unwrap().position, Some(100));
    }
}
