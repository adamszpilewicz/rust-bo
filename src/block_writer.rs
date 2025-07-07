use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::{SqlBlock, CREATE_TABLE_SUFFIX, CREATE_INDEX_SUFFIX, DELETE_INDEX_SUFFIX};

/// Writes each SQL block to a new file named e.g. "STEP_001_CREATE_MyTable.sql".
/// Returns the number of blocks written.
pub fn write_blocks_to_files(mut blocks: Vec<SqlBlock>, output_dir: &Path) -> io::Result<usize> {
    // Pre-compile small regexes for detection
    let create_table_re = regex::Regex::new(r"(?i)^CREATE\s+TABLE").unwrap();
    let create_index_re = regex::Regex::new(r"(?i)^CREATE\s+INDEX").unwrap();

    let mut step = 0usize;

    for block in blocks.drain(..) {
        step += 1;
        let step_str = format!("{:03}", step);
        let file_name = format!("STEP_{}_{}.sql", step_str, block.name);

        let final_path = output_dir.join(file_name);
        if block.lines.is_empty() {
            continue;
        }

        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&final_path)?;

        for line in block.lines {
            // Trim trailing whitespace for easier detection
            let mut out_line = line.trim_end().to_string();

            if create_table_re.is_match(&out_line) {
                // Append CREATE TABLE suffix if not already present
                if !out_line.contains("TABLESPACE BO_D_RW_D_02") {
                    out_line.push_str(CREATE_TABLE_SUFFIX);
                }
            } else if create_index_re.is_match(&out_line) {
                // Delete TABLESPACE BO_I_RW_D_01 if present
                out_line = out_line.replace(DELETE_INDEX_SUFFIX, "");
                // Append CREATE INDEX suffix if not already present
                if !out_line.contains("TABLESPACE BO_I_RW_D_02") {
                    out_line.push_str(CREATE_INDEX_SUFFIX);
                }
                
            }

            // Ensure a semicolon at the end, if not present
            if !out_line.ends_with(';') {
                out_line.push(';');
            }

            writeln!(f, "{}", out_line)?;
        }
    }
    Ok(step)
}
