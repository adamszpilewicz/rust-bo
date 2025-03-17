use std::error::Error;
use std::fs;
use std::path::PathBuf;

// Import our helper functions
use bo::args::parse_args;  // Parses config.yaml
use bo::concatenate::concatenate_files;
use bo::deduplicate::deduplicate_file;
use bo::gather_files;
use bo::split_sql::split_into_blocks;
use bo::block_writer::write_blocks_to_files;
use bo::mod_writer::write_mod_file;  // Import the new module
use bo::{ALL_FILE_NAME, DEDUP_FILE_NAME};

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Parse configuration (pattern, mod_file, output)
    let (pattern_str, mod_file_str, output_dir) = parse_args();

    // Ensure the output directory exists.
    fs::create_dir_all(&output_dir)?;

    // 2. Gather matching files
    let files = gather_files(&pattern_str)?;
    if files.is_empty() {
        eprintln!("No files matched {:?}", pattern_str);
        std::process::exit(1);
    }

    // 3. Concatenate files into "diff_result_ALL.txt"
    let all_txt_path = output_dir.join(ALL_FILE_NAME);
    concatenate_files(&files, &all_txt_path)?;

    // 4. Deduplicate lines into "diff_deduplicated.txt"
    let dedup_txt_path = output_dir.join(DEDUP_FILE_NAME);
    deduplicate_file(&all_txt_path, &dedup_txt_path)?;

    // 5. Split the deduplicated file into SQL blocks
    let blocks = split_into_blocks(&dedup_txt_path)?;

    // 6. Write each block and retrieve the step count.
    let step_count = write_blocks_to_files(blocks, &output_dir)?;

    // 7. Handle the mod_file as STEP_XXX_ALTER_MOD.sql
    if !mod_file_str.is_empty() {
        let mod_file_path = PathBuf::from(mod_file_str);
        write_mod_file(&mod_file_path, &output_dir, step_count)?;
    }

    // 8. Print final output information.
    println!("🎉 Done!");
    println!(" - Concatenated file: {}", all_txt_path.display());
    println!(" - Deduplicated file: {}", dedup_txt_path.display());
    println!(" - Final SQL files are in: {}", output_dir.display());

    Ok(())
}
