use std::error::Error;
use std::fs;

use bo::args::parse_args;
use bo::concatenate::concatenate_files;
use bo::deduplicate::deduplicate_file;
use bo::split_sql::split_into_blocks;
use bo::block_writer::write_blocks_to_files;
use bo::{ALL_FILE_NAME, DEDUP_FILE_NAME, STEP_FILE_FORMAT, CREATE_PREFIX};

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line args
    let (pattern_str, output_dir) = parse_args();

    // Ensure output dir exists
    fs::create_dir_all(&output_dir)?;

    // 1) Concatenate => diff_result_ALL.txt
    let all_txt_path = output_dir.join(ALL_FILE_NAME);
    let files = bo::gather_files(&pattern_str)?;
    concatenate_files(&files, &all_txt_path)?;

    // 2) Deduplicate => diff_deduplicated.txt
    let dedup_txt_path = output_dir.join(DEDUP_FILE_NAME);
    deduplicate_file(&all_txt_path, &dedup_txt_path)?;

    // 3) Split into blocks
    let blocks = split_into_blocks(&dedup_txt_path)?;

    // 4) Write each block
    write_blocks_to_files(blocks, &output_dir)?;

    println!("Done!");
    println!(" - {}", all_txt_path.display());
    println!(" - {}", dedup_txt_path.display());
    println!(
        " - {} (or {}...) files in {}",
        STEP_FILE_FORMAT,
        CREATE_PREFIX,
        output_dir.display()
    );

    Ok(())
}
