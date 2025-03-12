use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

pub fn concatenate_files(files: &[PathBuf], output_path: &Path) -> io::Result<()> {
    let out_file = File::create(output_path)?;
    let mut writer = BufWriter::new(out_file);

    // Now you can iterate over files without size errors
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
