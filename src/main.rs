use std::error::Error;
use std::fs;

// Import our helper functions and constants from our library modules.
use bo::args::parse_args; // New: loads configuration from config.yaml
use bo::concatenate::concatenate_files;
use bo::deduplicate::deduplicate_file;
use bo::gather_files;
use bo::split_sql::split_into_blocks;
use bo::block_writer::write_blocks_to_files;
use bo::{ALL_FILE_NAME, DEDUP_FILE_NAME};

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Parse configuration from config.yaml (with possible CLI overrides)
    //    This returns (pattern, mod_file, output_dir)
    let (pattern_str, mod_file_str, output_dir) = parse_args();

    // Ensure the output directory exists.
    fs::create_dir_all(&output_dir)?;

    // 2. Gather matching files
    let files = gather_files(&pattern_str)?;
    if files.is_empty() {
        eprintln!("No files matched {:?}", pattern_str);
        std::process::exit(1);
    }

    // 3. Concatenate all files into ALL_FILE_NAME (e.g., "diff_result_ALL.txt")
    let all_txt_path = output_dir.join(ALL_FILE_NAME);
    concatenate_files(&files, &all_txt_path)?;

    // 4. Deduplicate lines into DEDUP_FILE_NAME (e.g., "diff_deduplicated.txt")
    let dedup_txt_path = output_dir.join(DEDUP_FILE_NAME);
    deduplicate_file(&all_txt_path, &dedup_txt_path)?;

    // 5. Split the deduplicated file into SQL blocks
    let blocks = split_into_blocks(&dedup_txt_path)?;

    // 6. Write each block to separate files.
    //    write_blocks_to_files now returns the final step count.
    let step_count = write_blocks_to_files(blocks, &output_dir)?;

    // 7. If mod_file is specified (non-empty), read it and write its contents as a new step.
    if !mod_file_str.is_empty() {
        let mod_file_path = std::path::Path::new(&mod_file_str);
        let mod_contents = fs::read_to_string(mod_file_path)
            .expect("Failed to read mod_file; please check the path in config.yaml");
        let new_step = step_count + 1;
        let step_str = format!("{:03}", new_step);
        let mod_file_name = format!("STEP_{}_ALTER_MOD.sql", step_str);
        let final_mod_path = output_dir.join(mod_file_name);
        fs::write(&final_mod_path, mod_contents)
            .expect("Unable to write mod_file contents to final output");
    }

    // 8. Print final output information.
    println!("Done!");
    println!(" - Concatenated file: {}", all_txt_path.display());
    println!(" - Deduplicated file: {}", dedup_txt_path.display());
    println!(" - Final SQL files are in: {}", output_dir.display());

    Ok(())
}
