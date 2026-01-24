//! Error types for the mismall compression library
//!
//! This module defines a comprehensive error hierarchy that provides detailed
//! information about what went wrong during compression, decompression,
//! or archive operations.

use std::fmt;

/// Main error type for all mismall operations
#[derive(Debug)]
pub enum MismallError {
    /// Errors that occur during compression operations
    Compression(CompressionError),
    /// Errors that occur during decompression operations
    Decompression(DecompressionError),
    /// Errors that occur during archive operations
    #[cfg(feature = "archives")]
    Archive(ArchiveError),
    /// I/O related errors
    Io(std::io::Error),
    /// Invalid input parameters
    InvalidInput(String),
    /// Unsupported operation
    Unsupported(String),
}

impl fmt::Display for MismallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MismallError::Compression(e) => write!(f, "Compression error: {}", e),
            MismallError::Decompression(e) => write!(f, "Decompression error: {}", e),
            #[cfg(feature = "archives")]
            MismallError::Archive(e) => write!(f, "Archive error: {}", e),
            MismallError::Io(e) => write!(f, "I/O error: {}", e),
            MismallError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            MismallError::Unsupported(msg) => write!(f, "Unsupported operation: {}", msg),
        }
    }
}

impl std::error::Error for MismallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MismallError::Compression(e) => Some(e),
            MismallError::Decompression(e) => Some(e),
            #[cfg(feature = "archives")]
            MismallError::Archive(e) => Some(e),
            MismallError::Io(e) => Some(e),
            MismallError::InvalidInput(_) | MismallError::Unsupported(_) => None,
        }
    }
}

/// Errors that can occur during compression operations
#[derive(Debug)]
pub enum CompressionError {
    /// Failed to read input file
    InputRead(String),
    /// Failed to write output file
    OutputWrite(String),
    /// Encryption failed
    Encryption(String),
    /// Memory allocation failed
    Memory(String),
    /// Chunk size is invalid
    InvalidChunkSize(usize),
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionError::InputRead(msg) => write!(f, "Failed to read input: {}", msg),
            CompressionError::OutputWrite(msg) => write!(f, "Failed to write output: {}", msg),
            CompressionError::Encryption(msg) => write!(f, "Encryption failed: {}", msg),
            CompressionError::Memory(msg) => write!(f, "Memory error: {}", msg),
            CompressionError::InvalidChunkSize(size) => {
                write!(
                    f,
                    "Invalid chunk size: {}. Must be between 64KB and 1GB.",
                    size
                )
            }
        }
    }
}

impl std::error::Error for CompressionError {}

/// Errors that can occur during decompression operations
#[derive(Debug)]
pub enum DecompressionError {
    /// File format is invalid
    InvalidFormat(String),
    /// File version is not supported
    UnsupportedVersion(u8),
    /// Magic bytes don't match expected format
    InvalidMagic,
    /// File is encrypted but no password provided
    PasswordRequired,
    /// Provided password is incorrect
    InvalidPassword,
    /// Decryption failed
    Decryption(String),
    /// Data is corrupted or tampered
    CorruptedData(String),
    /// Checksum verification failed
    ChecksumMismatch,
    /// Failed to read compressed data
    InputRead(String),
    /// Failed to write decompressed data
    OutputWrite(String),
}

impl fmt::Display for DecompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecompressionError::InvalidFormat(msg) => write!(f, "Invalid file format: {}", msg),
            DecompressionError::UnsupportedVersion(ver) => {
                write!(
                    f,
                    "Unsupported file version: {}. Expected version {}.",
                    ver,
                    super::VERSION
                )
            }
            DecompressionError::InvalidMagic => {
                write!(f, "Invalid magic bytes - not a .small file")
            }
            DecompressionError::PasswordRequired => {
                write!(f, "File is encrypted - password required")
            }
            DecompressionError::InvalidPassword => write!(f, "Invalid password provided"),
            DecompressionError::Decryption(msg) => write!(f, "Decryption failed: {}", msg),
            DecompressionError::CorruptedData(msg) => {
                write!(f, "Data corruption detected: {}", msg)
            }
            DecompressionError::ChecksumMismatch => write!(f, "Checksum verification failed"),
            DecompressionError::InputRead(msg) => write!(f, "Failed to read input: {}", msg),
            DecompressionError::OutputWrite(msg) => write!(f, "Failed to write output: {}", msg),
        }
    }
}

impl std::error::Error for DecompressionError {}

/// Errors that can occur during archive operations
#[cfg(feature = "archives")]
#[derive(Debug)]
pub enum ArchiveError {
    /// Failed to create archive
    Creation(String),
    /// Failed to extract from archive
    Extraction(String),
    /// File not found in archive
    FileNotFound(String),
    /// Archive is corrupted
    Corrupted(String),
    /// Too many files in archive
    TooManyFiles(usize),
    /// Archive size exceeds limits
    TooLarge(u64),
}

#[cfg(feature = "archives")]
impl fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchiveError::Creation(msg) => write!(f, "Archive creation failed: {}", msg),
            ArchiveError::Extraction(msg) => write!(f, "Archive extraction failed: {}", msg),
            ArchiveError::FileNotFound(name) => write!(f, "File '{}' not found in archive", name),
            ArchiveError::Corrupted(msg) => write!(f, "Archive is corrupted: {}", msg),
            ArchiveError::TooManyFiles(count) => write!(f, "Too many files in archive: {}", count),
            ArchiveError::TooLarge(size) => write!(f, "Archive too large: {} bytes", size),
        }
    }
}

#[cfg(feature = "archives")]
impl std::error::Error for ArchiveError {}

// Conversion from std::io::Error
impl From<std::io::Error> for MismallError {
    fn from(err: std::io::Error) -> Self {
        MismallError::Io(err)
    }
}

// Conversion from Box<dyn std::error::Error>
impl From<Box<dyn std::error::Error>> for MismallError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        MismallError::InvalidInput(err.to_string())
    }
}

/// Result type for mismall operations
pub type Result<T> = std::result::Result<T, MismallError>;
