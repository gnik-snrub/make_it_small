//! Error condition tests
//!
//! Tests error handling, error contexts, and suggestions across all APIs.
//! Ensures proper error propagation and user-friendly error messages.

#[cfg(test)]
mod archive_errors;
#[cfg(test)]
mod compression_errors;
#[cfg(test)]
mod decompression_errors;
#[cfg(test)]
mod error_context_tests;
#[cfg(test)]
mod stream_errors;
