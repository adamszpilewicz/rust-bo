use glob::glob;
use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------
const DEFAULT_PATTERN: &str = "./input/diff_result_*";
const DEFAULT_OUTPUT: &str = "./output";
// Filenames for the concatenation & dedup phases
const ALL_FILE_NAME: &str = "diff_result_ALL.txt";
const DEDUP_FILE_NAME: &str = "diff_deduplicated.txt";

// Regex patterns
const ALTER_PATTERN: &str = r"(?i)^ALTER\s+TABLE\s+[A-Za-z0-9_]+\.(T\d+)_[A-Za-z0-9_]+";
const CREATE_PATTERN: &str = r"(?i)^CREATE\s+TABLE\s+([A-Za-z0-9_]+)\.([A-Za-z0-9_]+)";
const GRANT_REF_PATTERN: &str = r"(?i)^GRANT\s+REFERENCES\s+ON\s+[A-Za-z0-9_]+\.[A-Za-z0-9_]+";

// Block prefixes
const ALTER_PREFIX: &str = "ALTER_";
const CREATE_PREFIX: &str = "CREATE_";

// File format string for the final split files
const STEP_FILE_FORMAT: &str = "STEP_{}_{}.sql";

// ---------------------------------------------------------------------
// Data Structures
// ---------------------------------------------------------------------
#[derive(Debug)]
struct SqlBlock {
    name: String,
    lines: Vec<String>,
}

// ---------------------------------------------------------------------
// Main Entry
// ---------------------------------------------------------------------
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (pattern_str, output_dir) = parse_args();
    fs::create_dir_all(&output_dir)?;

    // 1) Gather matching files
    let mut files = Vec::new();
    for entry in glob(&pattern_str)? {
        match entry {
            Ok(path) => files.push(path),
            Err(e) => eprintln!("Warning: {}", e),
        }
    }
    if files.is_empty() {
        eprintln!("No files matched {:?}", pattern_str);
        std::process::exit(1);
    }

    // 2) Concatenate => diff_result_ALL.txt
    let all_txt_path = output_dir.join(ALL_FILE_NAME);
    concatenate_files(&files, &all_txt_path)?;

    // 3) Deduplicate => diff_deduplicated.txt
    let dedup_txt_path = output_dir.join(DEDUP_FILE_NAME);
    deduplicate_file(&all_txt_path, &dedup_txt_path)?;

    // 4) Split into blocks
    let blocks = split_into_blocks(&dedup_txt_path)?;

    // 5) Write each block
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

// ---------------------------------------------------------------------
// parse_args: read command-line flags:
//   --pattern=<someglob>  (defaults to DEFAULT_PATTERN)
//   --output=<somepath>   (defaults to DEFAULT_OUTPUT)
// Returns (pattern_string, output_directory_pathbuf)
// ---------------------------------------------------------------------
fn parse_args() -> (String, PathBuf) {
    let mut pattern_str = String::from(DEFAULT_PATTERN);
    let mut output_str = String::from(DEFAULT_OUTPUT);

    for arg in env::args().skip(1) {
        if let Some(stripped) = arg.strip_prefix("--pattern=") {
            pattern_str = stripped.to_string();
        } else if let Some(stripped) = arg.strip_prefix("--output=") {
            output_str = stripped.to_string();
        }
    }

    let output_dir = PathBuf::from(&output_str);
    (pattern_str, output_dir)
}

// ---------------------------------------------------------------------
// 1) Concatenate multiple files -> diff_result_ALL.txt
// ---------------------------------------------------------------------
fn concatenate_files(files: &[PathBuf], output_path: &Path) -> io::Result<()> {
    let out_file = File::create(output_path)?;
    let mut writer = BufWriter::new(out_file);

    for path in files {
        let file_name = path.to_string_lossy();
        let f = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Warning: could not open {}: {}", file_name, e);
                continue;
            }
        };
        let mut reader = BufReader::new(f);
        io::copy(&mut reader, &mut writer)?;
    }

    writer.flush()?;
    Ok(())
}

// ---------------------------------------------------------------------
// 2) Deduplicate lines -> "diff_deduplicated.txt"
// ---------------------------------------------------------------------
fn deduplicate_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let input = File::open(input_path)?;
    let reader = BufReader::new(input);

    let output = File::create(output_path)?;
    let mut writer = BufWriter::new(output);

    let mut seen = HashSet::new();
    for line_res in reader.lines() {
        let line = line_res?;
        if seen.insert(line.clone()) {
            writeln!(writer, "{}", line)?;
        }
    }
    writer.flush()?;
    Ok(())
}

// ---------------------------------------------------------------------
// 3) Split into blocks by scanning deduplicated lines top->bottom.
// ---------------------------------------------------------------------
fn split_into_blocks(input_path: &Path) -> io::Result<Vec<SqlBlock>> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    // Prepare regexes
    let re_alter = Regex::new(ALTER_PATTERN).map_err(to_io_error)?;
    let re_create = Regex::new(CREATE_PATTERN).map_err(to_io_error)?;
    let re_grant_ref = Regex::new(GRANT_REF_PATTERN).map_err(to_io_error)?;

    let mut blocks = Vec::new();
    let mut current_block = SqlBlock {
        name: String::new(),
        lines: Vec::new(),
    };

    let total = lines.len();
    let mut i = 0;

    while i < total {
        let line = lines[i].trim();

        // 1) Try to handle ALTER
        if let Some(new_i) =
            handle_alter_line(line, i, &lines, &mut current_block, &mut blocks, &re_alter)
        {
            i = new_i;
            continue;
        }

        // 2) Try to handle CREATE
        if let Some(new_i) = handle_create_line(
            line,
            i,
            &lines,
            total,
            &mut current_block,
            &mut blocks,
            &re_create,
            &re_grant_ref,
            &re_alter,
        ) {
            i = new_i;
            continue;
        }

        // Otherwise skip
        i += 1;
    }

    finalize_block(&mut current_block, &mut blocks);
    Ok(blocks)
}

// ---------------------------------------------------------------------
// 4) Write each block to "STEP_XXX_<blockname>.sql"
// ---------------------------------------------------------------------
fn write_blocks_to_files(mut blocks: Vec<SqlBlock>, output_dir: &Path) -> io::Result<()> {
    let mut step = 0usize;

    for block in blocks.drain(..) {
        step += 1;
        let step_str = format!("{:03}", step);

        // Either inline the format:
        let file_name = format!("STEP_{}_{}.sql", step_str, block.name);

        // Or call a helper function:
        // let file_name = step_file_name(&step_str, &block.name);

        let final_path = output_dir.join(file_name);
        if block.lines.is_empty() {
            continue;
        }

        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&final_path)?;
        for ln in block.lines {
            writeln!(f, "{}", ln)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------
// HELPER: handle_alter_line
// ---------------------------------------------------------------------
fn handle_alter_line(
    line: &str,
    i: usize,
    lines: &[String],
    current_block: &mut SqlBlock,
    blocks: &mut Vec<SqlBlock>,
    re_alter: &Regex,
) -> Option<usize> {
    if let Some(caps) = re_alter.captures(line) {
        let t_number = caps[1].to_string();
        finalize_block(current_block, blocks);
        let block_name = format!("{}{}", ALTER_PREFIX, t_number);
        let single_block = SqlBlock {
            name: block_name,
            lines: vec![lines[i].clone()],
        };
        blocks.push(single_block);
        Some(i + 1)
    } else {
        None
    }
}

// ---------------------------------------------------------------------
// HELPER: handle_create_line
// ---------------------------------------------------------------------
fn handle_create_line(
    line: &str,
    mut i: usize,
    lines: &[String],
    total: usize,
    current_block: &mut SqlBlock,
    blocks: &mut Vec<SqlBlock>,
    re_create: &Regex,
    re_grant_ref: &Regex,
    re_alter: &Regex,
) -> Option<usize> {
    if let Some(caps) = re_create.captures(line) {
        finalize_block(current_block, blocks);

        // Possibly attach preceding GRANT REFERENCES
        if i > 0 {
            let prev_line = lines[i - 1].trim();
            if re_grant_ref.is_match(prev_line) {
                current_block.lines.push(lines[i - 1].clone());
            }
        }

        let raw_table_name = caps[2].to_string();
        let short_name = short_table_name(&raw_table_name);
        current_block.name = format!("{}{}", CREATE_PREFIX, short_name);

        // add the CREATE line
        current_block.lines.push(lines[i].clone());
        i += 1;

        // gather subsequent lines
        while i < total {
            let next_line = lines[i].trim();
            if next_line.is_empty() {
                i += 1;
                continue;
            }
            if re_create.is_match(next_line) || re_alter.is_match(next_line) {
                break;
            }
            current_block.lines.push(lines[i].clone());
            i += 1;
        }

        finalize_block(current_block, blocks);
        Some(i)
    } else {
        None
    }
}

// ---------------------------------------------------------------------
// finalize_block
// ---------------------------------------------------------------------
fn finalize_block(current_block: &mut SqlBlock, blocks: &mut Vec<SqlBlock>) {
    if !current_block.lines.is_empty() {
        let new_block = SqlBlock {
            name: current_block.name.clone(),
            lines: current_block.lines.clone(),
        };
        blocks.push(new_block);
        current_block.name.clear();
        current_block.lines.clear();
    }
}

// ---------------------------------------------------------------------
// short_table_name
// ---------------------------------------------------------------------
fn short_table_name(full: &str) -> String {
    if let Some(pos) = full.find('_') {
        full[..pos].to_string()
    } else {
        full.to_string()
    }
}

// ---------------------------------------------------------------------
// to_io_error: convert regex::Error into io::Error
// ---------------------------------------------------------------------
fn to_io_error(e: regex::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}
