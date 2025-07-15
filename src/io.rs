
pub fn read_file(path: &str) -> Vec<u8> {
    match std::fs::read(path) {
        Ok(bytes) => {
            bytes
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            vec![]
        }
    }
}

pub fn write_file(file: Vec<u8>, path: &str) {
    let result = std::fs::write(path, file);
    match result {
        Ok(_) => {
            println!("File successfully created at {path}");
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}
