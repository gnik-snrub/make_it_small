#[cfg(feature = "cli")]
use console::style;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// File conflict resolution options
#[derive(Debug, Clone, Copy)]
pub enum ConflictResolution {
    Overwrite,
    Cancel,
    Backup,
}

/// Enhanced file conflict resolution with better UX
pub fn resolve_file_conflict(file_path: &Path) -> io::Result<Option<ConflictResolution>> {
    #[cfg(feature = "cli")]
    let _term = console::Term::stdout();

    if !file_path.exists() {
        return Ok(None);
    }

    // Get file info for better context
    let metadata = fs::metadata(file_path)?;
    let size = metadata.len();
    let modified = metadata
        .modified()
        .map(|dt| format!("{}", dt.elapsed().unwrap().as_secs()))
        .unwrap_or_else(|_| "unknown".to_string());

    println!();
    println!("File Conflict Detected");
    println!("│ File: {}", file_path.display());
    println!("│ Size: {} bytes", size);
    println!("│ Age: {} seconds ago", modified);
    println!("│ (O)verwrite existing file");
    println!("│ (B)ackup and overwrite");
    println!("│ (C)ancel operation");
    println!("{}", "─".repeat(50));

    loop {
        print!("Choose an option [O/B/C]: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim().to_lowercase().as_str() {
            "o" | "overwrite" => return Ok(Some(ConflictResolution::Overwrite)),
            "b" | "backup" => return Ok(Some(ConflictResolution::Backup)),
            "c" | "cancel" => return Ok(Some(ConflictResolution::Cancel)),
            _ => {
                println!("Invalid choice. Please try again.");
            }
        }
    }
}
