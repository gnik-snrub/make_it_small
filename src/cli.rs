use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::huffman::{decoder::decode, encoder::encode};

use crate::archive::{self, add_to_archive, create_archive, extract_archive};
use crate::flags;
use crate::headers::Headers;
use crate::progress::{OperationType, ProgressConfig, ProgressTracker};

#[derive(Parser, Debug)]
#[command(name = "Make It Small")]
#[command(about = "Streaming file compression and encryption tool", long_about = Some("A sophisticated file compressor with streaming architecture, AES-256-GCM encryption, and archive support. Handles arbitrarily large files with bounded memory usage."))]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Compress files with optional encryption")]
    Compress {
        #[clap(help = "Input file or directory to compress")]
        #[clap(index = 1)]
        name_in: String,

        #[clap(help = "Output basename (without .small extension)")]
        #[clap(index = 2)]
        name_out: Option<String>,

        #[clap(long_help = "Display compression ratio after completion")]
        #[clap(short, long, action = clap::ArgAction::SetTrue)]
        ratio: bool,

        #[clap(long_help = "Encrypt with AES-256-GCM using this password")]
        #[clap(short, long)]
        password: Option<String>,

        #[clap(
            long_help = "Memory chunk size for processing (default: 16MB, min: 64KB recommended)"
        )]
        #[clap(long, value_name = "SIZE")]
        chunk_size: Option<usize>,
    },
    #[command(about = "Decompress files with optional decryption")]
    Decompress {
        #[clap(help = "Input .small file to decompress")]
        name_in: String,

        #[clap(help = "Output filename (default: restore original name from header)")]
        name_out: Option<String>,

        #[clap(long_help = "Decrypt with AES-256-GCM using this password")]
        #[clap(short, long)]
        password: Option<String>,

        #[clap(
            long_help = "Memory chunk size for processing (default: 16MB, min: 64KB recommended)"
        )]
        #[clap(long, value_name = "SIZE")]
        chunk_size: Option<usize>,
    },
    #[command(about = "List contents of archive files")]
    List {
        #[clap(help = "Archive .small file to inspect")]
        name_in: String,
    },
    #[command(about = "Extract specific file from archive")]
    ExtractFile {
        #[clap(help = "Archive .small file containing the file")]
        name_in: String,

        #[clap(help = "Filename to extract from archive")]
        file_to_extract: String,

        #[clap(help = "Output filename for extracted file")]
        name_out: Option<String>,

        #[clap(
            long_help = "Memory chunk size for processing (default: 16MB, min: 64KB recommended)"
        )]
        #[clap(long, value_name = "SIZE")]
        chunk_size: Option<usize>,
    },
}

pub fn run_command() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = Cli::parse();
    match tokens {
        Cli {
            command:
                Some(Command::Compress {
                    name_in,
                    name_out,
                    ratio,
                    password,
                    chunk_size,
                }),
        } => {
            let actual_chunk_size = chunk_size.unwrap_or(crate::crypto::DEFAULT_CHUNK_SIZE);
            let input_path = Path::new(&name_in);
            let output_path_str = match name_out {
                Some(name) => format!("{name}.small"),
                None => format!("{}.small", name_in),
            };
            let output_path = Path::new(&output_path_str);

            if output_path.exists() {
                print!(
                    "Output file '{}' already exists. [O]verwrite, [A]dd to archive, or [C]ancel? ",
                    output_path_str
                );
                io::stdout().flush().unwrap();
                let mut choice = String::new();
                io::stdin().read_line(&mut choice).unwrap();

                match choice.trim().to_lowercase().as_str() {
                    "o" | "overwrite" => {
                        // Proceed to overwrite, logic is below
                    }
                    "a" | "add" => {
                        // 1. Prepare new content as a reader (either a single file or a temporary archive of a directory)
                        let mut new_content_reader: Box<File> = if input_path.is_dir() {
                            // Changed to mut Box<File>
                            let temp_archive_name = format!("{}.tmp_new_content", &output_path_str);
                            let temp_archive_path = Path::new(&temp_archive_name);
                            let temp_archive_file =
                                File::create(temp_archive_path).map_err(|e| {
                                    eprintln!(
                                        "Error creating temporary archive file '{}': {}",
                                        temp_archive_path.display(),
                                        e
                                    );
                                    Box::new(e) as Box<dyn std::error::Error>
                                })?;
                            let mut temp_archive_writer = BufWriter::new(temp_archive_file);
                            create_archive(
                                input_path,
                                password.as_deref(),
                                &mut temp_archive_writer,
                                actual_chunk_size,
                            )
                            .map_err(|e| {
                                eprintln!(
                                    "Error creating temporary archive of new directory: {}",
                                    e
                                );
                                e // create_archive returns Box<dyn Error>, so 'e' is already boxed
                            })?;
                            temp_archive_writer.flush()?;
                            Box::new(File::open(temp_archive_path).map_err(|e| {
                                eprintln!(
                                    "Error opening temporary archive file '{}': {}",
                                    temp_archive_path.display(),
                                    e
                                );
                                Box::new(e) as Box<dyn std::error::Error>
                            })?)
                        } else {
                            let temp_file_name = format!("{}.tmp_new_file", &output_path_str);
                            let temp_file_path = Path::new(&temp_file_name);
                            let temp_file = File::create(temp_file_path).map_err(|e| {
                                eprintln!(
                                    "Error creating temporary file '{}': {}",
                                    temp_file_path.display(),
                                    e
                                );
                                Box::new(e) as Box<dyn std::error::Error>
                            })?;
                            let mut temp_file_writer = BufWriter::new(temp_file);
                            let mut input_file_to_add =
                                BufReader::new(File::open(&name_in).map_err(|e| {
                                    eprintln!(
                                        "Error opening input file '{}' for add operation: {}",
                                        name_in, e
                                    );
                                    Box::new(e) as Box<dyn std::error::Error>
                                })?);
                            encode(
                                &mut input_file_to_add,
                                input_path.file_name().unwrap().to_str().unwrap(),
                                password.as_deref(),
                                &mut temp_file_writer,
                                actual_chunk_size,
                            )
                            .map_err(|e| {
                                eprintln!("Error compressing file for add operation: {}", e);
                                e // encode returns Box<dyn Error>, so 'e' is already boxed
                            })?;
                            temp_file_writer.flush()?;
                            Box::new(File::open(temp_file_path).map_err(|e| {
                                eprintln!(
                                    "Error opening temporary file '{}': {}",
                                    temp_file_path.display(),
                                    e
                                );
                                Box::new(e) as Box<dyn std::error::Error>
                            })?)
                        };

                        // 2. Prepare existing archive reader
                        let mut existing_archive_reader =
                            BufReader::new(File::open(output_path).map_err(|e| {
                                eprintln!(
                                    "Error opening existing archive '{}': {}",
                                    output_path.display(),
                                    e
                                );
                                Box::new(e) as Box<dyn std::error::Error>
                            })?);

                        // 3. Prepare temporary output writer
                        let temp_output_path_str = format!("{}.tmp", &output_path_str);
                        let temp_output_file =
                            File::create(&temp_output_path_str).map_err(|e| {
                                eprintln!(
                                    "Error creating temporary output file '{}': {}",
                                    temp_output_path_str, e
                                );
                                Box::new(e) as Box<dyn std::error::Error>
                            })?;
                        let mut temp_output_writer = BufWriter::new(temp_output_file);

                        // 4. Call add_to_archive
                        add_to_archive(
                            &mut existing_archive_reader,
                            new_content_reader.as_mut(),
                            &mut temp_output_writer,
                            actual_chunk_size,
                        )
                        .map_err(|e| {
                            eprintln!("Error adding to archive: {}", e);
                            e // add_to_archive returns Box<dyn Error>, so 'e' is already boxed
                        })?;
                        temp_output_writer.flush()?;

                        // 5. Replace original file with temporary output
                        fs::rename(&temp_output_path_str, output_path).map_err(|e| {
                            eprintln!(
                                "Error replacing original archive with updated archive: {}",
                                e
                            );
                            Box::new(e) as Box<dyn std::error::Error>
                        })?;

                        println!("File(s) added to archive '{}'", output_path.display());
                        return Ok(());
                    }
                    "c" | "cancel" => {
                        println!("Operation cancelled.");
                        return Ok(());
                    }
                    _ => {
                        eprintln!("Invalid choice. Aborting.");
                        return Ok(());
                    }
                }
            }

            // This is the original logic (for new files or overwrite)
            if input_path.is_dir() {
                let mut output_file = BufWriter::new(File::create(output_path).map_err(|e| {
                    eprintln!("Error creating archive file '{}': {}", output_path_str, e);
                    Box::new(e) as Box<dyn std::error::Error>
                })?);
                if let Err(e) = create_archive(
                    input_path,
                    password.as_deref(),
                    &mut output_file,
                    actual_chunk_size,
                ) {
                    eprintln!("Error creating archive: {}", e);
                    return Err(Box::new(e) as Box<dyn std::error::Error>); // create_archive returns Result<(), std::io::Error>
                }
                println!("Directory '{}' archived to '{}'", name_in, output_path_str);
            } else {
                let mut input_file = BufReader::new(File::open(&name_in).map_err(|e| {
                    eprintln!("Error opening input file '{}': {}", name_in, e);
                    Box::new(e) as Box<dyn std::error::Error>
                })?);
                let name = input_path.file_name().unwrap().to_str().unwrap();
                let mut output_file =
                    BufWriter::new(File::create(&output_path_str).map_err(|e| {
                        eprintln!("Error creating output file '{}': {}", output_path_str, e);
                        Box::new(e) as Box<dyn std::error::Error>
                    })?);

                let encode_result = encode(
                    &mut input_file,
                    name,
                    password.as_deref(),
                    &mut output_file,
                    actual_chunk_size,
                );

                let encode_info = match encode_result {
                    Ok(info) => info,
                    Err(e) => {
                        eprintln!("Error compressing file: {}", e);
                        return Err(e); // encode returns Box<dyn Error>, so 'e' is already boxed
                    }
                };

                if ratio {
                    let ratio = (encode_info.compressed_size as f64
                        / encode_info.original_size as f64)
                        * 100.0;
                    let rounded = (ratio * 10.0).round() / 10.0;
                    println!("Compression Ratio: {}%", rounded);
                }
                // No need to call write_file explicitly, encode already wrote to output_file
            }
        }
        Cli {
            command:
                Some(Command::Decompress {
                    name_in,
                    name_out,
                    password,
                    chunk_size,
                }),
        } => {
            let actual_chunk_size = chunk_size.unwrap_or(crate::crypto::DEFAULT_CHUNK_SIZE);
            let _input_path = Path::new(&name_in); // This is still here, but truly unused as name_in is used directly.

            let mut source_reader = BufReader::new(File::open(&name_in).map_err(|e| {
                eprintln!("Error opening input file '{}': {}", name_in, e);
                Box::new(e) as Box<dyn std::error::Error>
            })?);

            let header = match Headers::from_reader(&mut source_reader) {
                Ok(hdr) => hdr,
                Err(e) => {
                    eprintln!("Error parsing file header: {}", e);
                    return Err(e); // Headers::from_reader returns Box<dyn Error>, so 'e' is already boxed
                }
            };

            let decrypt_password_str: Option<String> = if let Some(p) = password {
                Some(p)
            } else if flags::is_encrypted(header.flags) {
                print!("File is encrypted. Please enter password: ");
                io::stdout().flush().unwrap();
                let mut password_input = String::new();
                io::stdin().read_line(&mut password_input).unwrap();
                Some(password_input.trim().to_string())
            } else {
                None
            };
            let decrypt_password = decrypt_password_str.as_deref();

            if flags::is_archive(header.flags) {
                let output_dir_name = match name_out {
                    Some(name) => name,
                    None => header.original_file_name.clone(),
                };
                let output_dir = Path::new(&output_dir_name);

                if let Err(e) = extract_archive(
                    header,
                    &mut source_reader,
                    output_dir,
                    decrypt_password,
                    actual_chunk_size,
                ) {
                    eprintln!("Error extracting archive: {}", e);
                    return Err(Box::new(e) as Box<dyn std::error::Error>); // extract_archive returns Result<(), std::io::Error>
                }
                println!(
                    "Archive '{}' extracted to directory '{}'",
                    name_in, output_dir_name
                );
            } else {
                let output_path_str = match name_out {
                    Some(name) => name,
                    None => header.original_file_name.clone(),
                };
                let output_path = Path::new(&output_path_str);

                let mut output_file = BufWriter::new(File::create(&output_path).map_err(|e| {
                    eprintln!(
                        "Error creating output file '{}': {}",
                        output_path.display(),
                        e
                    );
                    Box::new(e) as Box<dyn std::error::Error>
                })?);

                let _decode_info = decode(
                    header,
                    &mut source_reader,
                    decrypt_password,
                    &mut output_file,
                    actual_chunk_size,
                )
                .map_err(|e| {
                    eprintln!("Error decompressing file: {}", e);
                    e // decode returns Box<dyn Error>, so 'e' is already boxed
                })?;

                println!("File '{}' decompressed to '{}'", name_in, output_path_str);
            }
        }
        Cli { command: None } => {}
        Cli {
            command: Some(Command::List { name_in }),
        } => {
            let mut input_file = BufReader::new(File::open(&name_in).map_err(|e| {
                eprintln!("Error opening input file '{}': {}", name_in, e);
                Box::new(e) as Box<dyn std::error::Error>
            })?);
            match archive::list_contents(&mut input_file) {
                Ok(files) => {
                    if files.is_empty() {
                        println!("Archive '{}' is empty or not found.", name_in);
                    } else {
                        println!("Contents of '{}':", name_in);
                        for file_name in files {
                            println!("  {}", file_name);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error listing archive contents: {}", e);
                    return Err(Box::new(e) as Box<dyn std::error::Error>); // archive::list_contents returns Result<Vec<String>, std::io::Error>
                }
            }
        }
        Cli {
            command:
                Some(Command::ExtractFile {
                    name_in,
                    file_to_extract,
                    name_out,
                    chunk_size,
                }),
        } => {
            let actual_chunk_size = chunk_size.unwrap_or(crate::crypto::DEFAULT_CHUNK_SIZE);
            let mut input_file = BufReader::new(File::open(&name_in).map_err(|e| {
                eprintln!("Error opening archive file '{}': {}", name_in, e);
                Box::new(e) as Box<dyn std::error::Error>
            })?);

            let output_path = match name_out {
                Some(name) => PathBuf::from(name),
                None => PathBuf::from(&file_to_extract), // Default to file_to_extract if no output name given
            };

            if let Err(e) = archive::extract_file(
                &mut input_file,
                &file_to_extract,
                &output_path,
                None,
                actual_chunk_size,
            ) {
                eprintln!(
                    "Error extracting file '{}' from archive '{}': {}",
                    file_to_extract, name_in, e
                );
                return Err(Box::new(e) as Box<dyn std::error::Error>); // archive::extract_file returns Result<(), std::io::Error>
            }
            println!(
                "Successfully extracted '{}' from '{}' to '{}'",
                file_to_extract,
                name_in,
                output_path.display()
            );
        }
    }
    Ok(())
}
