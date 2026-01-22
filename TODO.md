# TODO

## Fix Compilation Errors

- [x] Add `#[derive(Debug)]` to `DecodeInfo` in `src/huffman/decoder.rs`.
- [x] Update calls to `decode` in `src/tests/huffman_tests/decoder_tests.rs` to pass the `Headers` struct.
- [x] Remove unused imports and variables across the project.
- [x] Fix `E0382: use of moved value: embedded_header` in `archive.rs`.

## Improve Code Quality

- [x] Address all warnings from `cargo test`.
- [x] Fix `tests::roundtrip_test::test_encrypted_file_roundtrip` (error message mismatch).
- [ ] Implement `HAS_CHECKSUM` flag integration.
- [ ] Implement `ENCRYPTED` flag integration for adding to archives.
- [ ] Implement checksum verification in `cli.rs`.

## Fix Archive/Header Related Test Failures

- [x] Fix `tests::archive_tests::tests::test_archive_roundtrip` (NUL byte/invalid UTF-8 in filename).
- [x] Fix `tests::archive_tests::tests::test_list_archive_contents` (returns empty list).
- [x] Fix `tests::archive_tests::tests::test_extract_single_file_from_archive` (file not found).
- [x] Fix `tests::archive_tests::tests::test_encrypted_archive_roundtrip` (invalid UTF-8 sequence in embedded header).
- [x] Fix `tests::archive_tests::tests::test_add_to_existing_archive` (No such file or directory).

## Implement Streaming

- [x] Refactor `cli.rs` Decompress command to stream initial compressed file loading.
- [x] Refactor `archive.rs` `extract_archive` to accept an already-opened `BufReader`.
- [x] Refactor `archive.rs` `create_archive` to stream encoded data directly to the writer, avoiding `all_encoded_files_bytes: Vec<u8>`.
- [x] Refactor `archive.rs` `add_to_archive` to stream existing and new content.
- [x] Refactor `cli.rs` Compress command `add` option to stream existing and new content.
- [x] Replace `io::read_file` and `io::write_file` usages with direct `File` or `BufReader`/`BufWriter` operations.

## Future Features

- [ ] Verify `STORED_RAW` flag implementation against `REFERENCE.md`.
- [ ] Verify the file format implementation against `REFERENCE.md`.