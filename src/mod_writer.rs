use std::fs;
use std::io;
use std::path::Path;

/// Writes the content of `mod_file` into a new SQL file named `STEP_XXX_ALTER_MOD.sql`.
///
/// - `mod_file_path`: Path to the `alter_mod.txt` file.
/// - `output_dir`: Directory where the final SQL file should be created.
/// - `step_count`: The last used step number. This function will increment it by 1.
///
/// Returns: `Ok(())` on success, or an `io::Error`.
pub fn write_mod_file(
    mod_file_path: &Path,
    output_dir: &Path,
    step_count: usize,
) -> io::Result<()> {
    if !mod_file_path.exists() {
        eprintln!("Warning: mod file {:?} not found, skipping.", mod_file_path);
        return Ok(()); // Skip processing if file does not exist.
    }

    let mod_contents = fs::read_to_string(mod_file_path)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Failed to read mod_file!"))?;

    // Our new step is step_count + 1
    let new_step = step_count + 1;
    let step_str = format!("{:03}", new_step);
    let mod_file_name = format!("STEP_{}_ALTER_MOD.sql", step_str);
    let final_mod_path = output_dir.join(mod_file_name);

    fs::write(&final_mod_path, mod_contents)
        .map_err(|_| io::Error::new(io::ErrorKind::WriteZero, "Failed to write mod_file!"))?;

    println!("✅ Created: {}", final_mod_path.display());
    Ok(())
}
