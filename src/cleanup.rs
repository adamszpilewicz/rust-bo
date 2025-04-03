use std::fs;
use std::io;
use std::path::Path;

/// Removes all files in the output directory before processing.
pub fn clean_output_directory(output_dir: &Path) -> io::Result<()> {
    if output_dir.exists() {
        for entry in fs::read_dir(output_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(&path)?;
            }
        }
        println!("🗑️  Cleaned output directory: {}", output_dir.display());
    } else {
        fs::create_dir_all(output_dir)?;
    }
    Ok(())
}
