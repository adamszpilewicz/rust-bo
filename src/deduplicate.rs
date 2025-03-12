use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// Deduplicate lines from `input_path` -> `output_path`.
pub fn deduplicate_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
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
