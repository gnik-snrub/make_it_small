# Performance Tips & Best Practices

This guide provides comprehensive performance optimization tips and best practices for using the mismall library effectively.

## 🚀 Quick Performance Checklist

- [ ] Choose appropriate chunk size for your system
- [ ] Use streaming API for large files
- [ ] Skip encryption when not needed
- [ ] Batch small files in archives
- [ ] Profile memory usage
- [ ] Test with your specific data types

## 💾 Memory Usage Optimization

### Chunk Size Selection

The most critical performance factor is choosing the right chunk size for your system and data:

| System RAM | Recommended Chunk Size | Use Case |
|------------|---------------------|-----------|
| 1-2 GB | 64KB - 256KB | Minimal memory footprint |
| 4-8 GB | 1MB - 16MB | Good balance (default: 16MB) |
| 16-32 GB | 32MB - 64MB | Large file processing |
| 64+ GB | 128MB - 1GB | Maximum performance |

```rust
// Low memory system
let result = CompressionBuilder::new("large_file.txt")
    .with_chunk_size(256 * 1024) // 256KB
    .compress()?;

// High-performance system
let result = CompressionBuilder::new("large_file.txt")
    .with_chunk_size(128 * 1024 * 1024) // 128MB
    .compress()?;
```

### Memory Usage Formula

```
Total Memory = Chunk Size + ~50KB overhead
```

**Example**: 16MB chunks = ~16.05MB total memory usage

## ⚡ Compression Performance Tips

### Data Type Considerations

Different data types compress differently with Huffman coding:

| Data Type | Expected Ratio | Recommendation |
|-----------|----------------|----------------|
| Text/Code | 25-40% | Excellent candidate |
| JSON/XML | 30-50% | Very good |
| Binary/Executables | 15-30% | Moderate |
| Images/Media | 95-105% | Often stored raw |
| Already Compressed | 100%+ | Store raw |

### Chunk Size Impact

```rust
// Test different chunk sizes for your data
let chunk_sizes = vec![64*1024, 256*1024, 1024*1024, 16*1024*1024];

for chunk_size in chunk_sizes {
    let start = std::time::Instant::now();
    let result = compress_stream(&mut reader, "test.dat", None, &mut output, chunk_size)?;
    let elapsed = start.elapsed();
    
    println!("Chunk size: {}KB, Time: {:?}, Ratio: {:.1}%", 
             chunk_size/1024, elapsed, result.compression_ratio);
}
```

## 🔒 Encryption Performance

### When to Use Encryption

| Scenario | Use Encryption | Reason |
|----------|----------------|---------|
| Private documents | ✅ Yes | Security critical |
| Temporary data | ❌ No | Performance matters |
| Public distribution | ✅ Yes | Integrity protection |
| Internal cache | ❌ No | Speed priority |

### Encryption Overhead

- **Performance**: ~10-20% slower compression/decompression
- **Size**: ~16 bytes per 16MB chunk + 28 bytes header
- **Memory**: No additional memory overhead

## 📁 Archive Optimization

### Batch Small Files

```rust
// Bad: Individual compression
for file in small_files {
    compress_file(file.path(), None)?; // Each file has overhead
}

// Good: Archive compression
let mut builder = ArchiveBuilder::new();
for file in small_files {
    let data = std::fs::read(file.path())?;
    builder = builder.add_file(file.path().to_str().unwrap(), data)?;
}
builder.build("combined.small")?; // Shared overhead
```

### Archive Size Optimization

- **Files < 1KB**: Avoid archiving (compression overhead > savings)
- **Files 1KB-10KB**: Batch in groups of 10-50 files
- **Files > 10KB**: Archive individually or in small groups

## 🔄 Streaming Best Practices

### Progressive Processing

```rust
use mismall::stream::{Compressor, Decompressor};
use std::io::BufReader;

// Process large files without loading entirely into memory
let input_file = std::fs::File::open("large_file.txt")?;
let output_file = std::fs::File::create("compressed.small")?;
let mut compressor = Compressor::new(output_file, "large_file.txt", None)?;

let mut reader = BufReader::new(input_file);
let mut buffer = [0u8; 8192]; // 8KB buffer

loop {
    let bytes_read = reader.read(&mut buffer)?;
    if bytes_read == 0 { break; }
    
    compressor.write_all(&buffer[..bytes_read])?;
}

compressor.finish()?; // Important: finalize compression
```

## 📊 Benchmarking Your Data

### Performance Testing Template

```rust
use std::time::Instant;
use mismall::{compress_stream, decompress_stream};

fn benchmark_compression(data: &[u8], chunk_size: usize) -> Result<(f64, f64)> {
    // Benchmark compression
    let comp_start = Instant::now();
    let mut reader = std::io::Cursor::new(data);
    let mut compressed = Vec::new();
    let comp_result = compress_stream(&mut reader, "benchmark", None, &mut compressed, chunk_size)?;
    let comp_time = comp_start.elapsed().as_secs_f64();
    
    // Benchmark decompression
    let decomp_start = Instant::now();
    let mut comp_reader = std::io::Cursor::new(&compressed);
    let mut decompressed = Vec::new();
    let decomp_result = decompress_stream(&mut comp_reader, None, &mut decompressed, chunk_size)?;
    let decomp_time = decomp_start.elapsed().as_secs_f64();
    
    // Verify correctness
    assert_eq!(data, &decompressed);
    
    Ok((comp_time, decomp_time))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_data = generate_test_data(10_000_000); // 10MB
    
    let chunk_sizes = vec![64*1024, 1024*1024, 16*1024*1024];
    
    for chunk_size in chunk_sizes {
        let (comp_time, decomp_time) = benchmark_compression(&test_data, chunk_size)?;
        
        let throughput = test_data.len() as f64 / comp_time / 1_000_000.0; // MB/s
        println!("Chunk: {}KB, Compression: {:.2} MB/s, Decompression: {:.2} MB/s",
                 chunk_size/1024, throughput, test_data.len() as f64 / decomp_time / 1_000_000.0);
    }
    
    Ok(())
}
```

## 🎯 Specific Optimization Strategies

### Text/Code Processing

```rust
// Optimize for source code and text
let result = CompressionBuilder::new("source_code.rs")
    .with_chunk_size(4 * 1024 * 1024) // 4MB chunks for good balance
    .compress()?;
```

### Binary Data Processing

```rust
// Optimize for binary/executable data
let result = CompressionBuilder::new("program.exe")
    .with_chunk_size(32 * 1024 * 1024) // Larger chunks for binary
    .without_encryption() // Skip encryption for speed
    .compress()?;
```

## ⚠️ Common Performance Pitfalls

### Avoid These Mistakes

1. **Too Small Chunks**: < 64KB causes excessive I/O operations
2. **Too Large Chunks**: > 1GB can cause system swapping
3. **Unnecessary Encryption**: 15-20% performance penalty
4. **Individual Small Files**: Each file has compression overhead
5. **Ignoring Progress**: No way to monitor long operations
6. **No Error Handling**: Can't recover from failures

### Performance Anti-Patterns

```rust
// ❌ Bad: Loading entire file into memory
let large_data = std::fs::read("huge_file.txt")?; // Can exhaust memory
let result = compress_from_bytes(large_data)?; // No streaming

// ✅ Good: Using streaming API
let mut file = std::fs::File::open("huge_file.txt")?;
let result = compress_stream(&mut file, "huge_file.txt", None, &mut output, 16*1024*1024)?;
```

## 🎓 Summary

Key takeaways for optimal mismall performance:

1. **Profile your specific data** - compression ratios vary widely
2. **Choose chunk sizes wisely** - balance memory vs. I/O efficiency  
3. **Use streaming for large files** - avoid memory exhaustion
4. **Batch small files** - reduce compression overhead
5. **Skip unnecessary encryption** - 15-20% performance cost
6. **Monitor progress** - essential for long operations

Following these guidelines will help you get the best performance from mismall while maintaining memory efficiency and reliability.