# TODO

## Streaming Refactoring (Further Optimization for Large Files)

This section outlines areas where entire file contents (or significant portions) are still loaded into memory during processing, preventing true end-to-end streaming for extremely large files (e.g., 4GB+). Refactoring these points would further reduce peak memory usage.

### `src/io.rs`

*   `pub fn read_file(path: &str) -> std::io::Result<Vec<u8>>`:
    *   **Issue:** Reads the entire file into a `Vec<u8>`.
    *   **Refactor Goal:** Eliminate direct usage or replace with streaming `File::open` and `std::io::copy` to a `writer`.
*   `pub fn write_file(file: Vec<u8>, path: &str)`:
    *   **Issue:** Takes `Vec<u8>` as input, implying the entire file content is already in memory.
    *   **Refactor Goal:** Eliminate direct usage or replace with functions accepting `Read` trait objects for streaming input.

### `src/huffman/encoder.rs`

*   **`payload_buffer: Vec<u8>` / `final_payload: Vec<u8>`**:
    *   **Issue:** Holds the entire compressed content after Huffman encoding.
    *   **Refactor Goal:** Modify the Huffman encoding process to stream directly to the output `writer` in blocks, without fully materializing the entire compressed payload in a `Vec<u8>`. This would require a block-based Huffman encoding and writing scheme.
*   **Raw Storage Path (`reader.read_to_end(&mut final_payload)?`)**:
    *   **Issue:** If the encoder decides to store data raw (uncompressed) and without encryption, the entire original file content is read into `final_payload: Vec<u8>`.
    *   **Refactor Goal:** Instead of `read_to_end`, stream the original file content directly to the output `writer` (or temporary file) if raw storage is chosen.
*   **Encryption Path (`crypto::encrypt` input/output)**:
    *   **Issue:** The entire (compressed or raw) content is passed to `crypto::encrypt` as `&[u8]` and the entire encrypted content is returned as `Vec<u8>`.
    *   **Refactor Goal:** Integrate block-based encryption (e.g., AES-GCM with stream cipher mode or chunked processing) directly within the encoding process, allowing encrypted chunks to be written to the output `writer` without buffering the entire encrypted file in memory.

### `src/huffman/decoder.rs`

*   **Decryption Path (`encrypted_payload_bytes: Vec<u8>`, `decrypted_payload: Vec<u8>`)**:
    *   **Issue:** If the file is encrypted, the entire encrypted payload is read into `encrypted_payload_bytes: Vec<u8>`, and the entire decrypted payload is then held in `decrypted_payload: Vec<u8>`.
    *   **Refactor Goal:** Implement block-based decryption, where encrypted chunks are read from the input `reader`, decrypted in smaller blocks, and immediately written to the output `writer`, without buffering the entire encrypted or decrypted payload in memory.

### `src/crypto.rs`

*   `pub fn encrypt(data: &[u8], ...) -> Result<(Vec<u8>, [u8; TAG_LEN]), Box<dyn std::error::Error>>`:
    *   **Issue:** Takes `data: &[u8]` and returns `Vec<u8>`, implying full in-memory processing of the entire plaintext and ciphertext.
    *   **Refactor Goal:** Modify to accept a `Read` trait object for plaintext and a `Write` trait object for ciphertext, processing data in fixed-size blocks.
*   `pub fn decrypt(ciphertext: &[u8], ...) -> Result<Vec<u8>, Box<dyn std::error::Error>>`:
    *   **Issue:** Takes `ciphertext: &[u8]` and returns `Vec<u8>`, implying full in-memory processing of the entire ciphertext and plaintext.
    *   **Refactor Goal:** Modify to accept a `Read` trait object for ciphertext and a `Write` trait object for plaintext, processing data in fixed-size blocks.
