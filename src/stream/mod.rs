//! Streaming utilities for stateful compression and decompression
//!
//! This module provides high-level streaming abstractions that maintain state
//! across multiple read/write operations, making it easy to work with
//! continuous data streams without loading entire files into memory.

pub mod compressor;
pub mod decompressor;

pub use compressor::Compressor;
pub use decompressor::Decompressor;

use std::io::{Read, Write};

/// Create a stream reader that decompresses data on the fly
///
/// This function returns a `Read` implementer that automatically decompresses
/// data as it's read from the underlying source.
///
/// # Arguments
///
/// * `source` - Source data stream (must implement `Read`)
/// * `password` - Optional password for decryption
///
/// # Examples
///
/// ```rust
/// use mismall::stream::stream_reader;
/// use std::fs::File;
///
/// let compressed_file = File::open("data.txt.small")?;
/// let mut reader = stream_reader(compressed_file, None);
///
/// let mut buffer = String::new();
/// reader.read_to_string(&mut buffer)?;
/// println!("Decompressed: {}", buffer);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn stream_reader<R: Read>(source: R, password: Option<&str>) -> impl Read {
    Decompressor::new(source, password)
}

/// Create a stream writer that compresses data on the fly
///
/// This function returns a `Write` implementer that automatically compresses
/// data as it's written, saving it to the underlying destination.
///
/// # Arguments
///
/// * `destination` - Destination stream (must implement `Write + Seek`)
/// * `filename` - Original filename for metadata
/// * `password` - Optional password for encryption
///
/// # Examples
///
/// ```rust
/// use mismall::stream::stream_writer;
/// use std::fs::File;
///
/// let output_file = File::create("compressed.txt.small")?;
/// let mut writer = stream_writer(output_file, "data.txt", None);
///
/// writer.write_all(b"Hello, world!")?;
/// writer.finish()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn stream_writer<W: Write + std::io::Seek>(
    destination: W,
    filename: &str,
    password: Option<&str>,
) -> impl Write {
    Compressor::new(destination, filename, password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};

    #[test]
    fn test_stream_reader_creation() {
        let data = b"test data";
        let cursor = Cursor::new(data);
        let _reader = stream_reader(cursor, None);
        // Basic test - just ensure it compiles and doesn't panic
    }

    #[test]
    fn test_stream_writer_creation() {
        let cursor = Cursor::new(Vec::new());
        let _writer = stream_writer(cursor, "test.txt", None);
        // Basic test - just ensure it compiles and doesn't panic
    }
}
