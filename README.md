# mismall — a tiny Huffman compressor

A minimalist file compressor and decompressor built around canonical Huffman coding.  
Designed to be easy to read, easy to test, and easy to reason about.

## Highlights
- **Only 3 dependencies:** [clap](https://crates.io/crates/clap) for the CLI, [rand](https://crates.io/crates/rand) for randomness, and [serde](https://crates.io/crates/serde) for serialization.
- Simple CLI with **`compress`** and **`decompress`** subcommands.
- Deterministic output; lossless round-trip verified with SHA-256 during benchmarks.
- On-disk container: a tiny header, the codebook, the encoded payload, and the original filename.
- Includes a **raw-store heuristic**: if compression would expand the file, the tool stores the raw bytes instead and marks it in the header.

## Install
```bash
cargo build --release
cp target/release/mismall ~/.local/bin/
```

## Usage
- Compress (prints ratio if you want it):
  ```bash
  mismall compress [-r] <INPUT> [OUTPUT_BASENAME]
  ```
  - If `OUTPUT_BASENAME` is omitted: output is `<INPUT>.small`.  
  - If provided: output is `<OUTPUT_BASENAME>.small`.

- Decompress:
  ```bash
  mismall decompress <INPUT.small> [OUTPUT_NAME]
  ```
  - If `OUTPUT_NAME` is omitted: restores the original filename from the header.

## How it works
1. Read the input file, count symbol frequencies, build a canonical Huffman codebook.
2. Emit a container with a header, the codebook, and the bit-packed payload.
3. On decode, reconstruct the codebook and expand the payload back to the original file.

---

## Performance Snapshot (your machine, your corpus)

Benchmarks were run with `hyperfine` (multiple runs, warmups) followed by SHA-256 round-trip checks.  
Filenames are anonymized by type/size.

### Text / Structured Data
- **HTML (~4.5 MiB)**  
  Ratio: ~73% (to ~3.3 MiB).  
  Encode: ~92 ms. Decode: ~80 ms.

- **Source file (~4.4 KiB)**  
  Ratio: ~63% (to ~2.8 KiB).  
  Times: sub-millisecond to ~1 ms. (Hyperfine warns about accuracy at this scale.)

### Small / Medium Binaries
- **Binary (~5.5 MiB)**  
  Ratio: ~82% (to ~4.5 MiB).  
  Encode: ~108 ms. Decode: ~99 ms.

- **Binary (~82 MiB)**  
  Ratio: ~80% (to ~65 MiB).  
  Encode: ~1.6 s. Decode: ~1.46 s.

### Already-Compressed / Mixed Media
- **Executable (~174 MiB)**  
  Ratio: ~100% (no change).  
  Encode: ~3.0 s. Decode: ~0.24 s.

- **PDF (~76 MiB)**  
  Ratio: ~99.5% (no change).  
  Encode: ~1.46 s. Decode: ~2.61 s.

- **MP4 video (~67 MiB)**  
  Ratio: ~100% (no change).  
  Encode: ~1.23 s. Decode: ~2.73 s.

- **JPEG (~58 KiB)**  
  Ratio: ~101% (slight expansion).  
  Encode/Decode: a few ms; hyperfine warns about accuracy.

### Integrity
- All tested files round-tripped **PASS** under SHA-256.

### Takeaways
- **Shines on text**: 20–35% savings.  
- **Flatlines on already-compressed media**: expected for Huffman-only.  
- **Decode is consistently fast**, often on par with or faster than encode.

---

## Limitations
- **Whole-file in memory:** large inputs load entirely into RAM. Very large files may stress memory.  
- **No container integrity field yet:** verification done externally.  
- **No encryption:** this tool is for compression only, until a dedicated crypto spec is added.

---

## Roadmap
- **Streaming I/O** — bounded-memory encode/decode.  
- **Archive mode** — pack multiple files with metadata into one `.small` container.  
- **Encryption system** — optional AEAD, with explicit KDF and threat model, in a separate spec.

---

## Examples
```bash
# compress with ratio
mismall compress -r document.txt

# decompress restoring original filename
mismall decompress document.txt.small

# decompress to a specific filename
mismall decompress picture.jpg.small restored.jpg
```

---

## Testing
Mismall ships with a full test suite that covers:
- Core Huffman encoding/decoding logic
- Serialization and deserialization of headers
- Raw-store heuristic (ensuring incompressible files are handled correctly)
- End-to-end compress/decompress round-trips

Run all tests with:
```bash
cargo test
```

---

## License
MIT — do whatever you want, just don't claim you wrote it.
