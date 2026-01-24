//! # mismall - Streaming Huffman Compression Library
//!
//! A sophisticated Rust library for file compression and decompression built around
//! canonical Huffman coding with streaming architecture. Designed to handle arbitrarily
//! large files with bounded memory usage and optional AES-256-GCM encryption.
//!
//! ## Features
//!
//! - **Streaming Architecture**: Bounded memory usage (16MB default) with chunked I/O
//! - **AES-256-GCM Encryption**: Optional password-based encryption with authenticated data integrity
//! - **Archive Support**: Pack multiple files into single `.small` containers with metadata
//! - **Memory Efficient**: Uses temporary files for intermediate processing, never loads entire files into RAM
//! - **Raw-Store Heuristic**: Automatically stores uncompressed data if compression would expand file size
//! - **Configurable Chunk Sizes**: Users can adjust memory usage from 64KB to 1GB+
//!
//! ## Quick Start
//!
//! ```rust
//! use mismall::{compress_file, decompress_file};
//!
//! // Simple compression
//! let compressed = compress_file("document.txt", None)?;
//! println!("File compressed successfully!");
//!
//! // Simple decompression
//! let decompressed = decompress_file("document.txt.small", None)?;
//! println!("File decompressed successfully!");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Advanced Usage
//!
//! ```rust
//! use mismall::compress::CompressionBuilder;
//!
//! let result = CompressionBuilder::new("large_video.mp4")
//!     .with_password("secret123")
//!     .with_chunk_size(64 * 1024 * 1024) // 64MB chunks
//!     .with_progress_callback(|progress| {
//!         println!("Progress: {}%", progress.percentage);
//!     })
//!     .compress()?;
//!
//! println!("Compressed {} bytes to {} bytes",
//!          result.original_size, result.compressed_size);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Archive Operations
//!
//! ```rust
//! use mismall::archive::{ArchiveBuilder, ArchiveExtractor};
//!
//! // Create archive
//! ArchiveBuilder::new()
//!     .add_file("doc1.pdf", b"PDF content")?
//!     .add_file("image.jpg", b"JPG content")?
//!     .with_password("archive_secret")
//!     .build("backup.small")?;
//!
//! // Extract from archive
//! ArchiveExtractor::new("backup.small")
//!     .with_password("archive_secret")
//!     .extract_file("doc1.pdf", "restored_doc.pdf")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Streaming Utilities
//!
//! ```rust
//! use mismall::stream::{Compressor, Decompressor, stream_reader, stream_writer};
//! use std::fs::File;
//!
//! // Streaming compression
//! let output_file = File::create("data.txt.small")?;
//! let mut compressor = stream_writer(output_file, "data.txt", None);
//! compressor.write_all(b"Hello, ")?;
//! compressor.write_all(b"world!")?;
//! compressor.finish()?;
//!
//! // Streaming decompression
//! let input_file = File::open("data.txt.small")?;
//! let mut reader = stream_reader(input_file, None);
//! let mut buffer = String::new();
//! reader.read_to_string(&mut buffer)?;
//! println!("Decompressed: {}", buffer);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Memory Usage Guidelines
//!
//! - **Low memory systems (1GB RAM)**: `chunk_size = 65536` (64KB)
//! - **Standard systems (8GB+ RAM)**: Default 16MB (16,777,216 bytes)
//! - **High-memory systems (32GB+ RAM)**: `chunk_size = 1073741824` (1GB)
//!
//! ## Feature Flags
//!
//! - `compression` (default): Compression and decompression functionality
//! - `archives` (default): Multi-file archive operations
//! - `encryption` (default): AES-256-GCM encryption support
//! - `cli`: Command-line interface (enables all other features)
//!
//! ## Error Handling
//!
//! All library functions return `Result<T, MismallError>` where `MismallError` provides
//! detailed error information with context for troubleshooting.
//!
//! # License
//!
//! MIT - do whatever you want, just don't claim you wrote it.

#[cfg(feature = "archives")]
pub mod compress;
pub mod decompress;
#[cfg(feature = "compression")]
pub mod error;
#[cfg(feature = "compression")]
pub mod progress;

// Re-export key types for convenience
#[cfg(feature = "compression")]
pub use error::{CompressionError, DecompressionError, MismallError};

// Re-export simple API functions
#[cfg(feature = "compression")]
pub use compress::{compress_file, compress_stream};
#[cfg(feature = "compression")]
pub use decompress::{decompress_file, decompress_stream};

// Re-export archive API functions
#[cfg(feature = "archives")]
pub use archive::{extract_archive, list_archive_contents};
#[cfg(feature = "archives")]
pub use archive::{ArchiveBuilder, ArchiveExtractor, ArchiveInfo, FileInfo};

// Re-export constants
pub use constants::{MAGIC_BYTES, VERSION};
pub use crypto::DEFAULT_CHUNK_SIZE;

// Re-export streaming utilities
#[cfg(feature = "compression")]
pub use stream::{stream_reader, stream_writer, Compressor, Decompressor};

// Async support (placeholder for future implementation)
#[cfg(all(feature = "compression", feature = "async"))]
pub mod async_stream;

// Legacy modules for internal use (not part of public API)
mod checksum;
#[cfg(feature = "cli")]
mod cli;
mod constants;
mod crypto;
mod flags;
mod headers;
mod huffman;
mod io;
pub mod tests;
mod ux;

// Archive operations module
#[cfg(feature = "archives")]
pub mod archive;

// Streaming utilities module
#[cfg(feature = "compression")]
pub mod stream;
