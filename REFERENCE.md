# Reference: Debugging Huffman Decoding and Archive Extraction Issues

## Problem Description

The primary issue causing `InvalidData` and `UnexpectedEof` errors, particularly in archive extraction, stemmed from a fundamental misunderstanding of the `decode` function's input expectations within `src/archive.rs`.

**Original Flaw:**
1.  The `create_archive` function correctly calculates the master header's `compressed_size` to include embedded headers and their payloads.
2.  During extraction (`extract_archive` and `extract_file`), the logic would:
    *   Read an `embedded_header` using `Headers::from_reader(reader)`. This advanced the main `reader` past the embedded header.
    *   Then, it would attempt to reconstruct a stream for the `decode` function by concatenating the `embedded_header.clone().to_bytes()` with the `compressed_data_buf` (read separately). This created a `Cursor` containing `[EMBEDDED_HEADER_BYTES, COMPRESSED_DATA]`.
3.  The `decode` function itself, however, internally called `Headers::from_reader` to parse the header from the *beginning* of its input stream.
4.  This resulted in the `decode` function effectively reading the embedded header twice. The first `Headers::from_reader` call in `extract_archive` positioned the main reader correctly, but the subsequent `Cursor` creation (with the prepended cloned header) caused `decode` to re-read the header, misinterpreting the actual compressed data as part of the header or simply skipping the true data. This led to parsing failures, `UnexpectedEof` (because `decode` consumed bytes it shouldn't have) or `InvalidData` errors.

## Proposed and Implemented Fix

The core of the fix was to refactor the `decode` function to accept an already-parsed `Headers` object, and then expect the provided `reader` to be positioned directly at the start of the compressed payload data.

### Changes in `src/huffman/decoder.rs`

*   **Modified `decode` function signature:**
    ```rust
    pub fn decode<R: Read, W: Write>(
        header: Headers, // Now accepts an already parsed Headers object
        reader: &mut R,
        decrypt_password: Option<&str>,
        writer: &mut W,
    ) -> Result<DecodeInfo, Box<dyn std::error::Error>>
    ```
*   **Removed internal header parsing:** The line `let header = Headers::from_reader(reader)?;` was removed from within the `decode` function.
*   **Corrected payload reader initialization:** For unencrypted data, the `payload_reader` is now initialized using `reader.take(header.compressed_size)` to ensure it only reads the actual compressed data after the header.

### Changes in `src/archive.rs`

*   **Updated `extract_archive` function:**
    *   The erroneous logic of creating a `full_file_bytes` buffer by cloning the header and prepending it to compressed data was removed.
    *   The `decode` function call was updated to directly pass the `embedded_header` (already parsed) and the main `reader` (which is correctly positioned at the start of the compressed payload after the `embedded_header` was read) as arguments:
        ```rust
                let _decode_info = decode(
                    embedded_header, // Pass the already parsed header
                    reader,          // Pass the main reader, which is correctly positioned
                    decrypt_password,
                    &mut output_file,
                ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        ```
*   **Updated `extract_file` function:**
    *   Applied the same fix as `extract_archive` to its `decode` call.

### Changes in `src/tests/roundtrip_test.rs`

*   **Added `use crate::headers::Headers;` import.**
*   **`roundtrip_file_compression` test:**
    *   Before calling `decode`, the test now explicitly reads the header from the `encoded_cursor` using `Headers::from_reader`.
    *   The `decode` call passes this read `header` and the `encoded_cursor` (now positioned at the payload).
    *   Corrected variable names from `encoded_output_buffer` to `compressed_output_buffer` and `decompressed_output_buffer` to `decoded_output_buffer` for consistency with the test's definitions.
*   **`test_encrypted_file_roundtrip` test:**
    *   Similarly, the test now explicitly reads the header before each `decode` call (for correct password, wrong password, and no password scenarios).
    *   The `decode` calls pass the respective `header` objects and the `Cursor`s.
    *   Corrected variable names like `decrypted_output_buffer_correct` to `decoded_output_buffer`.

### Changes in `src/cli.rs`

*   **`Decompress` command (single file decompression path):**
    *   Inside the `else` block for single file decompression, a new `single_file_header` is read from `input_file` using `Headers::from_reader(&mut input_file)`.
    *   This `single_file_header` is then passed as the first argument to the `decode` function, ensuring `input_file` is correctly positioned for `decode` to read the compressed data.

## Remaining Considerations / Known Limitations

*   **Checksum Verification in CLI:** The current streaming `decode` function writes directly to an output file. This makes it difficult to calculate the checksum of the decompressed content within the CLI application *after* decompression without re-reading the entire output file. For now, the CLI relies on the `decode_info.checksum` returned by the `decode` function, assuming `decode` would internally verify the checksum if it needed to. A more robust solution for client-side checksum verification of streaming decompressed output might involve modifying the `decode` function to return the decompressed `Vec<u8>` (going against streaming principles) or introducing a separate checksum verification pass.
*   **Unused Imports and Variables Warnings:** There are still several `unused import` and `unused variable` warnings (`padding_bits`, `embedded_header_start_pos`, `MOD_ADLER`, `serialize_tree`). These should be addressed for code cleanliness, either by using the variables/imports or removing them. These are non-critical warnings but indicate areas for minor cleanup.
