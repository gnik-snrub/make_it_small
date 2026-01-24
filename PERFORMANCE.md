# Performance Guide

This guide helps you optimize mismall performance for different use cases and hardware configurations.

## Memory Usage Configuration

### Chunk Size Selection

The `--chunk-size` parameter controls memory usage and affects performance characteristics:

| System RAM | Recommended Chunk Size | Use Case |
|--------------|----------------------|------------|
| 1-2 GB | 64KB - 256KB | Embedded systems, low-memory VMs |
| 4-8 GB | 1MB - 4MB | Standard laptops, desktops |
| 16-32 GB | 16MB (default) | General purpose, good balance |
| 64GB+ | 64MB - 256MB | High-performance workstations |
| 128GB+ | 256MB - 1GB | Data processing servers |

### Chunk Size Effects

- **Smaller chunks**: Lower memory usage, more temporary file I/O overhead
- **Larger chunks**: Higher memory usage, fewer I/O operations, better throughput

**Rule of thumb**: Use the largest chunk size that fits comfortably in available RAM.

## Performance by File Type

### Best Compression Ratios
1. **Text files** (HTML, JSON, source code): 20-35% reduction
2. **Structured data** (CSV, XML, logs): 25-40% reduction
3. **Source code**: 30-45% reduction

### No Compression Expected
1. **Already compressed** (JPEG, MP4, ZIP): Stored raw
2. **Encrypted data**: Stored raw
3. **Random binary**: Stored raw (heuristic prevents expansion)

## Hardware Optimization

### CPU Considerations
- **AES-NI support**: Modern CPUs accelerate AES-256-GCM encryption
- **Multiple cores**: Currently single-threaded, but you can run multiple instances
- **Cache size**: Larger chunks benefit from CPU cache efficiency

### Storage Considerations
- **SSD vs HDD**: Temporary file operations benefit from SSD speeds
- **Available space**: Ensure adequate temp space (~2x compressed file size for operations)

## Benchmarking

### Simple Performance Test
```bash
# Compress with timing
time mismall compress --chunk-size 16777216 large_file.txt

# Decompress with timing  
time mismall decompress --chunk-size 16777216 large_file.txt.small
```

### Memory Monitoring
```bash
# Monitor memory usage during operations
/usr/bin/time -v mismall compress huge_file.txt
```

## Troubleshooting Performance

### Slow Compression
1. **Increase chunk size** if RAM permits
2. **Check disk space** - low space causes temporary file thrashing
3. **Close other applications** to free RAM

### Out of Memory Errors
1. **Reduce chunk size** with `--chunk-size` flag
2. **Check for memory leaks** in other applications
3. **Use smaller chunks** for very large files

### Slow Encryption/Decryption
1. **Verify AES-NI support**: `grep aes /proc/cpuinfo`
2. **Update CPU microcode** if available
3. **Use hardware acceleration**: Modern Rust crypto uses CPU instructions

## Scaling Expectations

### Linear Scaling
- **Time**: Scales linearly with input size
- **Memory**: Constant (chunk_size + overhead)
- **Disk I/O**: Proportional to file size + temporary overhead

### Large File Handling
- **100MB files**: <5 seconds on modern hardware
- **1GB files**: <60 seconds on modern hardware  
- **10GB+ files**: Limited by disk speed, not CPU

### Archive Operations
- **Multi-file archives**: Time = sum of individual file times + metadata overhead
- **Extraction**: Constant time per file regardless of archive size
- **Listing**: Linear time with number of files in archive

## Optimization Tips

### For Maximum Speed
```bash
# High-memory system (32GB+ RAM)
mismall compress --chunk-size 1073741824 input_file
```

### For Minimum Memory Usage
```bash
# Low-memory system (1GB RAM)  
mismall compress --chunk-size 65536 input_file
```

### For Large Archives
```bash
# Process in stages to manage temporary space
mismall compress --chunk-size 4194304 directory/ archive_name
```

## Real-World Examples

### Web Server Log Compression
```bash
# Compress today's logs (8GB RAM, 64GB SSD)
mismall compress --chunk-size 33554432 access.log
# Expected: 70% compression ratio, ~2 minutes
```

### Database Backup Encryption  
```bash
# Encrypt large database backup (64GB RAM)
mismall compress -p 'backup_password' --chunk-size 268435456 database.dump
# Expected: Minor compression, strong encryption, ~10 minutes
```

### Source Code Archiving
```bash
# Archive entire project directory
mismall compress --chunk-size 16777216 my_project/ project_archive
# Expected: 35% compression, includes all metadata
```