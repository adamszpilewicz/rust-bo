use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use crate::args::InstallConfig;

/// Writes the `install.TXT` file and includes the footer SQL file.
///
/// - `output_dir`: Directory where the SQL files are stored.
/// - `install_config`: YAML configuration for `install.TXT`
pub fn write_install_file(output_dir: &Path, install_config: &InstallConfig) -> io::Result<()> {
    let install_path = output_dir.join(&install_config.install_file_name);
    let mut install_file = File::create(&install_path)?;

    println!("🔹 Writing install file: {}", install_path.display());

    // 🏁 Write header lines from YAML
    for header in &install_config.header_lines {
        writeln!(install_file, "{}", header)?;
    }

    // 📂 Fetch all existing SQL files in the output directory
    let mut sql_files: Vec<String> = fs::read_dir(output_dir)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let file_name = entry.file_name();
            let file_str = file_name.to_string_lossy().into_owned();
            if file_str.starts_with("STEP_") && file_str.ends_with(".sql") {
                Some(file_str)
            } else {
                None
            }
        })
        .collect();

    // 🔢 Sort files to maintain correct order
    sql_files.sort();
    println!("📂 Found SQL files: {:?}", sql_files);

    // ✅ Start step counter from `002` (since headers are `000` and `001`)
    let mut step_counter = 2;

    // 📝 Write SQL execution steps from existing SQL files
    for file_name in &sql_files {
        let formatted_line = format!(
            "{}|{}",
            format!("{:03}", step_counter),  // Correctly formatted step number
            install_config.sql_execution_format.replace("{}", file_name)
        );
        writeln!(install_file, "{}", formatted_line)?;
        println!("✅ Added to install.TXT: {}", formatted_line);

        step_counter += 1; // Increment AFTER writing each file
    }

    // ✅ Append footer SQL file if configured
    if let Some(ref footer_template) = install_config.footer_line {
        let footer_step_str = format!("{:03}", step_counter); // Footer uses the next step

        // 🔍 Extract filename from last pipe (`|`) in footer template
        let footer_sql_name = footer_template
            .split('|') // Split the string by `|`
            .last() // Take the last segment
            .unwrap_or("")
            .replace("{}", &footer_step_str); // Replace step number

        println!("📌 Extracted footer SQL file name: {}", footer_sql_name);

        let footer_line = footer_template.replace("{}", &footer_step_str);
        writeln!(install_file, "{}", footer_line)?;
        println!("✅ Added footer entry to install.TXT: {}", footer_line);

        // 📄 Write the footer SQL file
        if let Some(ref footer_sql_content) = install_config.footer_sql_content {
            let footer_sql_path = output_dir.join(&footer_sql_name);
            let mut footer_sql_file = File::create(&footer_sql_path)?;

            println!("✅ Creating footer SQL file: {}", footer_sql_path.display());

            footer_sql_file.write_all(footer_sql_content.as_bytes())?;
            println!("✅ Footer SQL file created: {}", footer_sql_path.display());
        }
    }

    println!("✅ Created install file: {}", install_path.display());
    Ok(())
}
