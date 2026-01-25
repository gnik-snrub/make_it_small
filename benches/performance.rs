use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mismall::{compress::CompressionBuilder, compress_file, compress_stream, decompress_file};
use std::io::{Cursor, Write};
use std::time::Duration;
use tempfile::NamedTempFile;

// Test data sizes for benchmarking
const TEST_SIZES: &[usize] = &[1024, 10240, 102400, 1024000]; // 1KB, 10KB, 100KB, 1MB

fn create_test_data(size: usize) -> Vec<u8> {
    vec![0u8; size]
}

fn create_test_file(size: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    let data = create_test_data(size);
    file.write_all(&data).unwrap();
    file
}

fn benchmark_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");

    for &size in TEST_SIZES {
        let test_file = create_test_file(size);
        let test_path = test_file.path();

        // Library compression
        group.bench_with_input(
            BenchmarkId::new("library", format!("{}KB", size / 1024)),
            &size,
            |b, _| {
                b.iter(|| {
                    let result = compress_file(test_path.to_str().unwrap(), None).unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_compression_stream(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_stream");

    for &size in TEST_SIZES {
        let data = create_test_data(size);

        // Stream compression
        group.bench_with_input(
            BenchmarkId::new("library", format!("{}KB", size / 1024)),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut reader = Cursor::new(&data);
                    let mut writer = Cursor::new(Vec::new());
                    let result = compress_stream(
                        &mut reader,
                        "test.txt",
                        None,
                        &mut writer,
                        16 * 1024 * 1024,
                    )
                    .unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // Test with different chunk sizes to verify memory remains bounded
    let chunk_sizes = &[65536, 131072, 262144, 524288]; // 64KB, 128KB, 256KB, 512KB

    for &chunk_size in chunk_sizes {
        let test_file = create_test_file(1024000); // 1MB test file
        let test_path = test_file.path();

        group.bench_with_input(
            BenchmarkId::new(
                "streaming_compression",
                format!("chunk_{}KB", chunk_size / 1024),
            ),
            &chunk_size,
            |b, &chunk_size| {
                b.iter(|| {
                    let result = CompressionBuilder::new(test_path.to_str().unwrap())
                        .with_chunk_size(chunk_size)
                        .compress()
                        .unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_large_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_files");
    group.measurement_time(Duration::from_secs(15)); // Reduced time for faster testing

    let large_sizes = &[10485760]; // 10MB (reduced for faster testing)

    for &size in large_sizes {
        let test_file = create_test_file(size);
        let test_path = test_file.path();

        group.bench_with_input(
            BenchmarkId::new("large_compression", format!("{}MB", size / 1048576)),
            &size,
            |b, _| {
                b.iter(|| {
                    let result = compress_file(test_path.to_str().unwrap(), None).unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_compression,
    benchmark_compression_stream,
    benchmark_memory_usage,
    benchmark_large_files
);
criterion_main!(benches);
