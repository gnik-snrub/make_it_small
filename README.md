# mismall - Streaming Huffman Compression Library

[![Crates.io](https://img.shields.io/crates/v/mismall)](https://crates.io/crates/mismall)
[![Documentation](https://docs.rs/mismall/badge.svg)](https://docs.rs/mismall)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A sophisticated Rust library for file compression and decompression built around canonical Huffman coding with streaming architecture. Designed to handle arbitrarily large files with bounded memory usage and optional AES-256-GCM encryption.

## 🚀 Library Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
mismall = "2.0"
```

## Highlights
- **Streaming Architecture:** Bounded memory usage (16MB default) with chunked I/O for unlimited file size support
- **AES-256-GCM Encryption:** Optional password-based encryption with authenticated data integrity
- **Archive Support:** Pack multiple files into single `.small` containers with metadata
- **Memory Efficient:** Uses temporary files for intermediate processing, never loads entire files into RAM
- **Raw-Store Heuristic:** Automatically stores uncompressed data if compression would expand file size
- **Configurable Chunk Sizes:** Users can adjust memory usage from 64KB to 1GB+ with `--chunk-size` flag
- **Deterministic Output:** Lossless round-trip verified with SHA-256 during processing

### Basic Library Usage

```rust
use mismall::{compress_stream, decompress_stream};
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create test data
    let input_data = b"Hello, world! This is test data for compression.";
    std::fs::write("test.txt", input_data)?;
    
    // Compress using stream API
    let mut reader = Cursor::new(input_data);
    let mut compressed = Vec::new();
    let result = mismall::compress_stream(&mut reader, "test.txt", None, &mut compressed, 1024 * 1024)?;
    
    println!("Compressed {} -> {} bytes ({:.1}% ratio)", 
             result.original_size, result.compressed_size, result.compression_ratio);
    
    // Save compressed data
    std::fs::write("test.txt.small", compressed)?;
    
    // Decompress the file
    let compressed_data = std::fs::read("test.txt.small")?;
    let mut compressed_reader = Cursor::new(compressed_data);
    let mut decompressed = Vec::new();
    let result = mismall::decompress_stream(&mut compressed_reader, None, &mut decompressed, 1024 * 1024)?;
    
    println!("Decompressed {} bytes", result.original_size);
    
    Ok(())
}
```

## 📦 Feature Flags

- `compression` (default): Compression and decompression functionality
- `archives` (default): Multi-file archive operations  
- `encryption` (default): AES-256-GCM encryption support
- `cli`: Command-line interface (enables all other features)

```toml
[dependencies]
mismall = { version = "2.0", default-features = false, features = ["compression", "encryption"] }
```

## 🎯 Core Library APIs

### Simple API
- [`compress_stream()`] - Compress data streams with custom settings
- [`decompress_stream()`] - Decompress data streams with custom settings

### Builder API  
- [`CompressionBuilder`] - Advanced compression with options
- [`DecompressionBuilder`] - Advanced decompression with options
- [`ArchiveBuilder`] - Create multi-file archives
- [`ArchiveExtractor`] - Extract from archives with options

### Streaming API
- [`stream_reader()`] - Read from compressed streams
- [`stream_writer()`] - Write to compressed streams  
- [`Compressor`] - Stateful streaming compression
- [`Decompressor`] - Stateful streaming decompression

## 🛠️ Library Examples

The `examples/` directory contains comprehensive library examples:

- `simple_compress.rs` - Basic compression and decompression
- `advanced_compression.rs` - Compression with encryption and custom settings
- `archive_operations.rs` - Multi-file archive creation and extraction
- `streaming.rs` - Real-time streaming compression/decompression
- `performance.rs` - Performance comparison and benchmarks

Run examples with:

```bash
cargo run --example simple_compress
cargo run --example advanced_compression
cargo run --example archive_operations
cargo run --example streaming
cargo run --example performance
```

## 📈 Performance Tips

For comprehensive performance optimization guidance, see [**PERFORMANCE.md**](PERFORMANCE.md):

- Memory usage optimization for different system configurations
- Chunk size selection strategies
- Data type-specific recommendations
- Encryption performance considerations
- Streaming best practices
- Benchmarking templates
- Common pitfalls to avoid

## 🔧 Error Handling

All library functions return `Result<T, MismallError>` where `MismallError` provides detailed error information with context for troubleshooting.

```rust
match mismall::compress_stream(&mut reader, "test.txt", None, &mut output, 1024 * 1024) {
    Ok(result) => println!("Success: {} bytes compressed", result.compressed_size),
    Err(e) => eprintln!("Compression failed: {}", e),
}
```

---

## CLI Tool Usage

The mismall library also includes a command-line interface. Install and use as follows:

## Install

### Single File Operations
- **Compress (with optional encryption and ratio display):**
  ```bash
  mismall compress [-r] [-p PASSWORD] [--chunk-size SIZE] <INPUT> [OUTPUT_BASENAME]
  ```
  - If `OUTPUT_BASENAME` is omitted: output is `<INPUT>.small`
  - If provided: output is `<OUTPUT_BASENAME>.small`
  - `--chunk-size`: Memory usage (default 16MB, min 64KB recommended)
  - `-p`: Optional password for AES-256-GCM encryption

- **Decompress:**
  ```bash
  mismall decompress [-p PASSWORD] [--chunk-size SIZE] <INPUT.small> [OUTPUT_NAME]
  ```
  - If `OUTPUT_NAME` is omitted: restores original filename from header
  - `--chunk-size`: Memory usage for decryption operations

### Archive Operations
- **Create archive from directory:**
  ```bash
  mismall compress [-r] [-p PASSWORD] [--chunk-size SIZE] <DIRECTORY> [ARCHIVE_NAME]
  ```

- **List archive contents:**
  ```bash
  mismall list <ARCHIVE.small>
  ```

- **Extract from archive:**
  ```bash
  mismall extract-file [-p PASSWORD] [--chunk-size SIZE] <ARCHIVE.small> <FILENAME> [OUTPUT_NAME]
  ```

### Memory Usage Guidelines
- **Low memory systems (1GB RAM):** `--chunk-size 65536` (64KB)
- **Standard systems (8GB+ RAM):** Default 16MB (16,777,216 bytes)
- **High-memory systems (32GB+ RAM):** `--chunk-size 1073741824` (1GB)

## How it works
1. **Pass 1:** Stream input file in configurable chunks to compute symbol frequencies and checksum
2. **Codebook Generation:** Build canonical Huffman tree and generate optimal code table
3. **Pass 2:** Stream input again, encoding data using bit-level packing with 4KB buffers
4. **Encryption (optional):** Apply AES-256-GCM with chunked processing and per-chunk authentication
5. **Archive Creation:** Combine multiple compressed files with metadata into single container
6. **Decoding:** Reverse process with streaming decryption and bit-level expansion

## Performance Characteristics

### Memory Usage
- **Bounded:** Maximum memory usage = `chunk-size` + small overhead (~50KB)
- **Scalable:** Handles arbitrarily large files with constant memory footprint
- **Temporary Storage:** Uses OS temporary files for intermediate processing

### Compression Performance
- **Text Files:** 20-35% size reduction, linear time complexity
- **Source Code:** 25-40% size reduction, fast encoding/decoding
- **Already-Compressed Media:** Stored raw (no expansion), minimal overhead

### Encryption Performance
- **AES-256-GCM:** Hardware-accelerated on modern CPUs
- **Per-Chunk Authentication:** Detect corruption early in the stream
- **Zero-Knowledge Security:** PBKDF2 key derivation with random salt

## Performance Snapshot (Intel i7, 16GB RAM)

### Text / Structured Data
- **HTML (~4.5 MiB)**  
  Ratio: 73% (to 3.3 MiB)  
  Encode: 92 ms. Decode: 80 ms.

- **Source file (~4.4 KiB)**  
  Ratio: 63% (to 2.8 KiB)  
  Times: sub-millisecond

### Small / Medium Binaries
- **Binary (~5.5 MiB)**  
  Ratio: 82% (to 4.5 MiB)  
  Encode: 108 ms. Decode: 99 ms.

- **Binary (~82 MiB)**  
  Ratio: 80% (to 65 MiB)  
  Encode: 1.6 s. Decode: 1.46 s.

### Archive Operations
- **Multi-file archive:** Linear scaling with total compressed size
- **Extraction:** Constant time per file, regardless of archive size
- **Encryption overhead:** ~16 bytes per 16MB chunk + 28 bytes header

### Encryption Performance
- **AES-256-GCM:** ~500 MB/s on modern CPUs with hardware acceleration
- **Memory overhead:** Configurable chunk size (default 16MB)
- **Authentication:** Per-chunk tags enable early corruption detection

### Integrity
- **All tested files round-tripped PASS** under SHA-256 verification
- **Chunk-level authentication:** Detects corruption during streaming
- **Memory bounds:** No buffer overflows or integer overflows in 66 tests

## Limitations
- **Streaming I/O Required:** Not designed for in-memory only operations (feature, not bug)
- **Huffman-Only Compression:** Less effective on already-compressed media than DEFLATE/LZ77
- **No Parallel Processing:** Single-threaded for simplicity and determinism

## Testing
Mismall ships with a comprehensive test suite (66 tests) covering:
- **Core Logic:** Huffman encoding/decoding with streaming architecture
- **Cryptographic Operations:** Key derivation, encryption, decryption, authentication
- **Archive Management:** Multi-file operations and metadata handling
- **Error Handling:** Corrupted data, wrong passwords, edge cases
- **Memory Safety:** Bounded memory usage under all conditions
- **I/O Operations:** Bit-level reading/writing with proper padding
- **Integration:** End-to-end compress/decompress/extract workflows

Run all tests with:
```bash
cargo test
```

## Examples
```bash
# Basic compression with ratio
mismall compress -r document.txt

# Compressed with encryption and custom chunk size
mismall compress -p mypassword --chunk-size 8388608 large_video.mp4 encrypted_archive.small

# Decompress with password
mismall decompress -p mypassword encrypted_archive.small

# Create archive from directory
mismall compress project/ project_archive

# Extract specific file from archive
mismall extract-file project_archive.small src/main.rs main_backup.rs

# List archive contents
mismall list project_archive.small
```

## License
MIT — do whatever you want, just don't claim you wrote it.