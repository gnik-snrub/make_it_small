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
    println!("{}", style("File Conflict Detected").yellow().bold());
    println!("┌─────────────────────────────────────────");
    println!("│ File: {}", style(file_path.display()).cyan());
    println!("│ Size: {} bytes", style(size).dim());
    println!("│ Age: {} seconds ago", style(modified).dim());
    println!("├─────────────────────────────────────────");

    println!(
        "{} {}",
        style("[O]").green().bold(),
        style("Overwrite existing file").white()
    );
    println!(
        "{} {}",
        style("[B]").blue().bold(),
        style("Create backup and overwrite").white()
    );
    println!(
        "{} {}",
        style("[C]").red().bold(),
        style("Cancel operation").white()
    );
    println!("└─────────────────────────────────────────");

    print!("{}", style("Your choice [O/B/C]: ").yellow().bold());
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    match choice.trim().to_lowercase().as_str() {
        "o" | "overwrite" => Ok(Some(ConflictResolution::Overwrite)),
        "b" | "backup" => {
            // Create backup with timestamp
            let backup_path = file_path.with_extension(format!(
                "{}.backup",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ));

            fs::rename(file_path, &backup_path)?;
            println!(
                "✓ Backed up existing file to: {}",
                style(backup_path.display()).green()
            );
            Ok(Some(ConflictResolution::Overwrite))
        }
        "c" | "cancel" => {
            println!("{}", style("Operation cancelled by user").yellow());
            Ok(Some(ConflictResolution::Cancel))
        }
        _ => {
            println!("{}", style("Invalid choice. Please try again.").red());
            // Recursively ask again
            resolve_file_conflict(file_path)
        }
    }
}

/// Enhanced error message display with suggestions
pub fn display_error_with_suggestions(error: &str) {
    let _term = console::Term::stdout();

    println!();
    println!("{}", style("❌ Error").red().bold());
    println!("{}", style("─".repeat(50)).red());

    // Split error message and display with formatting
    for line in error.lines() {
        println!("│ {}", style(line).white());
    }

    println!("{}", style("─".repeat(50)).red());
    println!("{}", style("Press Enter to continue...").dim());
    io::stdin().read_line(&mut String::new()).ok();
}
