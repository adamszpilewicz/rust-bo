use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::SqlBlock;

/// Writes each SQL block to a new file named e.g. "STEP_001_CREATE_MyTable.sql".
pub fn write_blocks_to_files(mut blocks: Vec<SqlBlock>, output_dir: &Path) -> io::Result<()> {
    let mut step = 0usize;

    for mut block in blocks.drain(..) {
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

        for ln in block.lines.drain(..) {
            writeln!(f, "{}", ln)?;
        }
    }

    Ok(())
}
