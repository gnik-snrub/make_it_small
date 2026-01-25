//! Error context tests
//!
//! Tests error context enhancement, suggestions, and error trait implementations.
//! Verifies that errors provide helpful information to users.

use mismall::error::{ErrorContext, ErrorExt, MismallError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new("test_operation")
            .with_file_path("test_file.txt")
            .with_chunk_size(1024);

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.file_path, Some("test_file.txt".to_string()));
        assert_eq!(context.chunk_size, Some(1024));
    }

    #[test]
    fn test_error_context_with_suggestion() {
        let context =
            ErrorContext::new("test_operation").with_suggestion("Try again", "Temporary failure");

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.suggestion, Some("Try again".to_string()));
        assert_eq!(
            context.suggestion_message,
            Some("Temporary failure".to_string())
        );
    }

    #[test]
    fn test_error_ext_on_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");

        let enhanced = io_error.with_context(ErrorContext::new("read_file"));

        match enhanced {
            MismallError::Io {
                error: _,
                context: Some(ctx),
                suggestion: _,
            } => {
                // Expected: Error should have context attached
                let ctx = context.unwrap();
                assert_eq!(ctx.operation, "read_file");
                assert_eq!(ctx.file_path, None);
            }
            other => panic!("Expected Io error with context, got {:?}", other),
        }
    }

    #[test]
    fn test_error_ext_with_suggestion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");

        let enhanced = io_error.with_suggestion("Check file permissions");

        match enhanced {
            MismallError::Io {
                error: _,
                context: _,
                suggestion: Some(sugg),
            } => {
                // Expected: Error should have suggestion attached
                assert_eq!(sugg.message, "Check file permissions");
                assert_eq!(sugg.suggestion_message, Some("Check file permissions"));
            }
            other => panic!("Expected Io error with suggestion, got {:?}", other),
        }
    }

    #[test]
    fn test_error_context_serialization() {
        let context = ErrorContext::new("test").with_custom_context("key", "value");

        let roundtrip = serde_json::to_string(&context).unwrap();
        let restored: ErrorContext = serde_json::from_str(&roundtrip).unwrap();

        assert_eq!(restored.operation, context.operation);
        assert_eq!(restored.custom_context, context.custom_context);
    }
}
