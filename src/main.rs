use std::error::Error;
use std::fs;
use std::path::PathBuf;

// Import modules
use bo::args::parse_args;
use bo::concatenate::concatenate_files;
use bo::deduplicate::deduplicate_file;
use bo::gather_files;
use bo::split_sql::split_into_blocks;
use bo::block_writer::write_blocks_to_files;
use bo::mod_writer::write_mod_file;
use bo::install_writer::write_install_file;
use bo::cleanup::clean_output_directory;

use bo::{ALL_FILE_NAME, DEDUP_FILE_NAME};

fn main() -> Result<(), Box<dyn Error>> {
    let (config, output_dir) = parse_args();

    // ✅ Ensure output directory exists and clean it before running
    clean_output_directory(&output_dir)?;

    fs::create_dir_all(&output_dir)?;

    let files = gather_files(&config.pattern)?;
    if files.is_empty() {
        eprintln!("No files matched {:?}", config.pattern);
        std::process::exit(1);
    }

    let all_txt_path = output_dir.join(ALL_FILE_NAME);
    concatenate_files(&files, &all_txt_path)?;

    let dedup_txt_path = output_dir.join(DEDUP_FILE_NAME);
    deduplicate_file(&all_txt_path, &dedup_txt_path)?;

    let blocks = split_into_blocks(&dedup_txt_path)?;

    let step_count = write_blocks_to_files(blocks, &output_dir)?;

    if !config.mod_file.is_empty() {
        let mod_file_path = PathBuf::from(&config.mod_file);
        write_mod_file(&mod_file_path, &output_dir, step_count)?;
    }

    // ✅ Generate install.TXT using YAML configuration
    write_install_file(&output_dir, &config.install_config)?;

    println!("🎉 Done!");
    println!(" - Final SQL files and install.TXT are in: {}", output_dir.display());

    Ok(())
}
