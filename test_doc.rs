//! Test for one doc example

fn main() {
    let result = test_result();
    assert!(result.is_ok());
}

fn test_result() -> Result<(), Box<dyn std::error::Error>> {
    use mismall::archive::ArchiveBuilder;

    let builder = ArchiveBuilder::new().add_file("document.txt", b"Hello, world!")?;

    Ok(())
}
