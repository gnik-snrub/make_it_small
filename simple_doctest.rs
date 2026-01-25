```rust
use mismall::archive::ArchiveBuilder;

let builder = ArchiveBuilder::new()
    .add_file("document.txt", b"Hello, world!")?;
```