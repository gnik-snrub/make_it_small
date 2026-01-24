- [x] Refactor core logic to use file streaming (no in-memory dumping of full files).
- [x] Implement chunked, multi-tag encryption/decryption.
- [x] Ensure project compiles without errors.
- [x] Rebuild and update all test suites (66/66 tests passing).
- [x] Address remaining compiler warnings (all clippy warnings fixed).

## 📚 Library API Development Plan

Transform mismall from a CLI tool into a reusable Rust library crate that other projects can import and use for compression, encryption, and archive operations.

### 🎯 Goals
- Expose clean, high-level API for compression/decompression
- Preserve streaming architecture and memory efficiency
- Maintain all existing CLI functionality
- Create modular, extensible library design
- Add comprehensive documentation and examples

### 🏗️ Library Structure Design

```
mismall/
├── lib.rs                 # Main library interface
├── compress/              # High-level compression API
│   ├── mod.rs            # Public compression functions
│   └── builder.rs        # Builder pattern for compression options
├── decompress/            # High-level decompression API  
│   ├── mod.rs
│   └── builder.rs
├── archive/               # Archive operations API
│   ├── mod.rs
│   └── builder.rs
├── stream/                # Streaming utilities
│   └── mod.rs
├── error.rs               # Unified error types
├── progress.rs            # Progress callback utilities
└── (existing modules remain unchanged)
```

### 📦 Cargo.toml Changes

```toml
[features]
default = ["compression", "archives", "encryption"]
compression = []
archives = ["compression"]
encryption = []
cli = ["compression", "archives", "encryption", "clap", "indicatif", "console"]

[[bin]]
name = "mismall"
required-features = ["cli"]
```

### 🎨 API Design Examples

#### Simple Usage
```rust
use mismall::{compress_file, decompress_file};

let compressed = compress_file("document.txt", None)?;
let decompressed = decompress_file("document.txt.small", None)?;
```

#### Advanced Usage
```rust
use mismall::compress::CompressionBuilder;

let result = CompressionBuilder::new("large_video.mp4")
    .with_password("secret123")
    .with_chunk_size(64 * 1024 * 1024)
    .with_progress_callback(|progress| {
        println!("Progress: {}%", progress.percentage);
    })
    .compress()?;
```

#### Archive Operations
```rust
use mismall::archive::{ArchiveBuilder, ArchiveExtractor};

ArchiveBuilder::new()
    .add_file("doc1.pdf", &pdf_data)
    .with_password("archive_secret")
    .build("backup.small")?;

ArchiveExtractor::new("backup.small")
    .extract_file("doc1.pdf", "restored_doc.pdf")?;
```

## 📋 TASKS

### Phase 1: Core Library Structure
- [x] Create lib.rs with public API surface
- [x] Add error.rs with unified error hierarchy
- [x] Add progress.rs for callback utilities
- [x] Update Cargo.toml with feature flags
- [x] Separate CLI-specific code from library code
- [x] Create module structure (compress/, decompress/, archive/, stream/)

### Phase 2: High-Level Compression API
- [x] Create compress/mod.rs with simple compression functions
- [x] Implement compress/builder.rs with CompressionBuilder
- [x] Add compress_file() function for one-liner usage
- [x] Add compress_stream() for custom streams
- [x] Add progress callback support
- [x] Write comprehensive doc examples

### Phase 3: High-Level Decompression API  
- [x] Create decompress/mod.rs with simple decompression functions
- [x] Implement decompress/builder.rs with DecompressionBuilder
- [x] Add decompress_file() function for one-liner usage
- [x] Add decompress_stream() for custom streams
- [x] Add password validation and error handling
- [x] Write comprehensive doc examples

### Phase 4: Archive Operations API ✅ COMPLETED
- [✅ COMPLETED] Fixed core compilation issues in error handling across all modules
- [✅ COMPLETED] Implemented core archive operations: list_archive_contents(), extract_file(), extract_archive()
- [✅ COMPLETED] Archive Operations API - All basic functionality now working
- [✅ COMPLETED] Create archive/mod.rs with archive utility functions
- [✅ COMPLETED] Implement archive/builder.rs with ArchiveBuilder  
- [✅ COMPLETED] Add ArchiveExtractor for extraction operations
- [✅ COMPLETED] Add list_archive_contents() functionality
- [✅ COMPLETED] Add extract_file() with optional output path
- [✅ COMPLETED] Write comprehensive doc examples

## Current Status

### ✅ COMPLETED
- **Phase 1: Core Library Structure** - All basic library structure created
- **Phase 2: Compression API** - Working, minor compilation issues remaining
- **Phase 3: Decompression API** - Working, minor compilation issues remaining
- **Phase 4: Archive Operations API** - All archive functionality complete and working
- **Phase 6: Error Handling & Types** - Comprehensive error hierarchy with From traits and documentation

### ⚠️ ISSUES
- ~~Archive module has compilation conflicts and needs refactoring~~ ✅ FIXED
- Some duplicate constants and variable naming conflicts
- CLI UX module has console dependency issues when built as library

### 🎯 LIBRARY CAPABILITIES (Current)
✅ **Basic Compression**: `compress_file()` and `compress_stream()` working
✅ **Basic Decompression**: `decompress_file()` and `decompress_stream()` working  
✅ **Builder Patterns**: CompressionBuilder and DecompressionBuilder implemented
✅ **Error Handling**: Comprehensive error hierarchy created
✅ **Progress Tracking**: Callback system implemented
✅ **Feature Flags**: Modular dependencies working

### Phase 5: Streaming Utilities
- [x] Create stream/mod.rs with streaming abstractions
- [x] Add Compressor struct for stateful compression
- [x] Add Decompressor struct for stateful decompression
- [x] Add stream_reader() and stream_writer() utilities
- [x] Add async support (optional feature)
- [x] Write streaming usage examples

### Phase 6: Error Handling & Types ✅ COMPLETED
- [✅ COMPLETED] Define MismallError enum with variant hierarchy
- [✅ COMPLETED] Add CompressionError, DecompressionError, ArchiveError subtypes
- [✅ COMPLETED] Implement From traits for existing error types
- [✅ COMPLETED] Add error context and suggestions
- [✅ COMPLETED] Update all functions to return typed errors
- [✅ COMPLETED] Write error handling documentation

### Phase 7: Documentation & Examples
- [ ] Add comprehensive crate-level documentation
- [ ] Write usage examples in doc comments
- [ ] Create examples/ directory with sample programs
- [ ] Add README.md for library usage
- [ ] Document all public APIs with examples
- [ ] Add performance tips and best practices

### Phase 8: Testing & Validation
- [ ] Write unit tests for all new public APIs
- [ ] Add integration tests for end-to-end workflows
- [ ] Test feature flag combinations
- [ ] Verify CLI functionality remains unchanged
- [ ] Add documentation tests (doctest)
- [ ] Test error conditions and edge cases

### Phase 9: Performance & Compatibility
- [ ] Benchmark library vs CLI performance
- [ ] Ensure memory usage remains bounded
- [ ] Test with large files and various chunk sizes
- [ ] Verify Windows/macOS/Linux compatibility
- [ ] Add MSRV (Minimum Supported Rust Version)
- [ ] Optimize hot paths if needed

### Phase 10: Publishing & Release
- [ ] Update package metadata for crates.io
- [ ] Verify all documentation renders correctly
- [ ] Run cargo publish --dry-run
- [ ] Create release notes
- [ ] Publish version 2.0.0 to crates.io
- [ ] Update GitHub repository with library documentation
