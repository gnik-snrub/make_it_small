# TODO

## Fix Compilation Errors

- [x] Add `#[derive(Debug)]` to `DecodeInfo` in `src/huffman/decoder.rs`.
- [x] Update calls to `decode` in `src/tests/huffman_tests/decoder_tests.rs` to pass the `Headers` struct.
- [ ] Remove unused imports and variables across the project.
- [x] Fix `E0382: use of moved value: embedded_header` in `archive.rs`.

## Improve Code Quality

- [ ] Address all warnings from `cargo test`.
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

## Future Features

- [ ] Verify `STORED_RAW` flag implementation against `REFERENCE.md`.
- [ ] Verify the file format implementation against `REFERENCE.md`.
