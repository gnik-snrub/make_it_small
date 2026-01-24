use std::fmt;

/// Enhanced error types with user-friendly messages
#[derive(Debug)]
pub enum MismallError {
    /// File I/O related errors with context
    IoError(String, std::io::Error),

    /// Encryption/decryption related errors
    EncryptionError(String),

    /// Invalid file format errors
    InvalidFormat(String),

    /// Memory/Chunk size related errors  
    MemoryError(String),

    /// Compression/Decompression errors
    CompressionError(String),

    /// Archive related errors
    ArchiveError(String),
}

impl fmt::Display for MismallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MismallError::IoError(context, io_err) => {
                write!(
                    f,
                    "{}\n\n💡 Suggestions:\n{}",
                    context,
                    get_io_suggestions(io_err)
                )
            }
            MismallError::EncryptionError(msg) => {
                write!(
                    f,
                    "{}\n\n💡 Suggestions:\n{}",
                    msg,
                    get_encryption_suggestions(msg)
                )
            }
            MismallError::InvalidFormat(msg) => {
                write!(
                    f,
                    "{}\n\n💡 Suggestions:\n{}",
                    msg,
                    get_format_suggestions(msg)
                )
            }
            MismallError::MemoryError(msg) => {
                write!(
                    f,
                    "{}\n\n💡 Suggestions:\n{}",
                    msg,
                    get_memory_suggestions(msg)
                )
            }
            MismallError::CompressionError(msg) => {
                write!(
                    f,
                    "{}\n\n💡 Suggestions:\n{}",
                    msg,
                    get_compression_suggestions(msg)
                )
            }
            MismallError::ArchiveError(msg) => {
                write!(
                    f,
                    "{}\n\n💡 Suggestions:\n{}",
                    msg,
                    get_archive_suggestions(msg)
                )
            }
        }
    }
}

impl std::error::Error for MismallError {}

/// Convert std::io::Error to MismallError with context
pub fn io_error(context: &str, err: std::io::Error) -> MismallError {
    MismallError::IoError(context.to_string(), err)
}

/// Create encryption error with user-friendly message
pub fn encryption_error(msg: &str) -> MismallError {
    MismallError::EncryptionError(msg.to_string())
}

/// Create format error with user-friendly message
pub fn format_error(msg: &str) -> MismallError {
    MismallError::InvalidFormat(msg.to_string())
}

/// Create memory error with user-friendly message
pub fn memory_error(msg: &str) -> MismallError {
    MismallError::MemoryError(msg.to_string())
}

/// Create compression error with user-friendly message
pub fn compression_error(msg: &str) -> MismallError {
    MismallError::CompressionError(msg.to_string())
}

/// Create archive error with user-friendly message
pub fn archive_error(msg: &str) -> MismallError {
    MismallError::ArchiveError(msg.to_string())
}

/// Get user-friendly suggestions for I/O errors
fn get_io_suggestions(io_err: &std::io::Error) -> String {
    match io_err.kind() {
        std::io::ErrorKind::NotFound => "• Check if the file path is correct\n\
             • Ensure the file exists and is accessible\n\
             • Verify you have read permissions\n\
             • Try using absolute paths if using relative paths"
            .to_string(),
        std::io::ErrorKind::PermissionDenied => "• Check file/directory permissions\n\
             • Try running with appropriate privileges\n\
             • Ensure the file is not in use by another program\n\
             • Verify the target directory is writable"
            .to_string(),
        std::io::ErrorKind::StorageFull => "• Free up disk space on the target drive\n\
             • Check available space before large operations\n\
             • Consider using a different drive/partition\n\
             • Clean temporary files in /tmp"
            .to_string(),
        std::io::ErrorKind::UnexpectedEof => "• The file may be corrupted or truncated\n\
             • Verify the file integrity if downloaded\n\
             • Check if the operation was interrupted\n\
             • Try decompressing a different copy if available"
            .to_string(),
        _ => "• Ensure the file is not locked by another program\n\
             • Check available disk space\n\
             • Verify network connection if accessing remote files\n\
             • Try the operation again"
            .to_string(),
    }
}

/// Get user-friendly suggestions for encryption errors
fn get_encryption_suggestions(msg: &str) -> String {
    if msg.contains("password") || msg.contains("decrypt") {
        "• Double-check the password - it's case-sensitive\n\
         • Try copying and pasting the password to avoid typos\n\
         • Ensure you're using the correct password for this file\n\
         • Check if the file was encrypted with a different password"
            .to_string()
    } else if msg.contains("corrupted") || msg.contains("tag") {
        "• The file may be corrupted during transfer\n\
         • Verify the file integrity with checksum if available\n\
         • Try re-downloading or re-transferring the file\n\
         • Check if the file was partially encrypted/decrypted"
            .to_string()
    } else {
        "• Ensure the file was actually encrypted\n\
         • Check if AES-256-GCM is supported on your system\n\
         • Verify the file is not corrupted\n\
         • Try processing a different file to test encryption"
            .to_string()
    }
}

/// Get user-friendly suggestions for format errors
fn get_format_suggestions(msg: &str) -> String {
    if msg.contains(".small") {
        "• Ensure you're decompressing a valid .small file\n\
         • Check if the file was created by this version of mismall\n\
         • Try decompressing with the --verbose flag for more details\n\
         • Verify the file header is not corrupted"
            .to_string()
    } else {
        "• Check if the file type is supported\n\
         • Ensure the file is not corrupted\n\
         • Try using a different file to test the operation\n\
         • Verify the file has a valid format"
            .to_string()
    }
}

/// Get user-friendly suggestions for memory errors
fn get_memory_suggestions(msg: &str) -> String {
    if msg.contains("chunk") || msg.contains("memory") {
        "• Reduce the --chunk-size parameter (try 8MB or 4MB)\n\
         • Close other applications to free up memory\n\
         • Check available RAM before large operations\n\
         • Try processing smaller files first"
            .to_string()
    } else {
        "• Increase available memory by closing other programs\n\
         • Use smaller chunk sizes with --chunk-size flag\n\
         • Check system memory usage\n\
         • Restart the application to clear memory leaks"
            .to_string()
    }
}

/// Get user-friendly suggestions for compression errors
fn get_compression_suggestions(msg: &str) -> String {
    if msg.contains("tree") || msg.contains("frequency") {
        "• Check if the input file is empty or corrupted\n\
         • Try compressing a different file to test the algorithm\n\
         • Ensure the file contains compressible data\n\
         • Verify file permissions are correct"
            .to_string()
    } else if msg.contains("buffer") || msg.contains("overflow") {
        "• Reduce the --chunk-size parameter\n\
         • Try processing a smaller file\n\
         • Check for corrupted input data\n\
         • Ensure adequate system resources"
            .to_string()
    } else {
        "• Verify the input file is valid and readable\n\
         • Try with a smaller chunk size\n\
         • Check available disk space\n\
         • Ensure the file is not locked"
            .to_string()
    }
}

/// Get user-friendly suggestions for archive errors
fn get_archive_suggestions(msg: &str) -> String {
    if msg.contains("extract") || msg.contains("not found") {
        "• List archive contents with 'mismall list' command\n\
         • Check exact filename (case-sensitive)\n\
         • Verify the file exists in the archive\n\
         • Try extracting with a different filename"
            .to_string()
    } else if msg.contains("corrupted") {
        "• Verify the archive file is not corrupted\n\
         • Try re-creating the archive\n\
         • Check if the archive transfer was complete\n\
         • Test with a backup copy if available"
            .to_string()
    } else {
        "• Ensure you have sufficient permissions for archive operations\n\
         • Check available disk space\n\
         • Verify the archive file is valid\n\
         • Try listing archive contents first"
            .to_string()
    }
}
