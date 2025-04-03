use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Represents the parsed configuration from `config.yaml`.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub pattern: String,
    pub mod_file: String,
    pub output: String,
    pub install_config: InstallConfig,
}

/// Struct to store the parsed YAML configuration for install.TXT
#[derive(Debug, Deserialize)]
pub struct InstallConfig {
    pub install_file_name: String,
    pub header_lines: Vec<String>,
    pub sql_execution_format: String,
    pub footer_line: Option<String>,  // ✅ Ensure this is an Option<String>
    pub footer_sql_content: Option<String>, // ✅ Ensure this is an Option<String>
}



/// Reads `config.yaml` and parses the values.
pub fn parse_args() -> (Config, PathBuf) {
    let config_file = "config.yaml";

    // Read the YAML file
    let yaml_str = fs::read_to_string(config_file)
        .expect(&format!("Failed to read {}", config_file));

    // Parse YAML using serde_yaml
    let parsed_config: Config = serde_yaml::from_str(&yaml_str)
        .expect("Invalid YAML format");

    let output_dir = PathBuf::from(&parsed_config.output);
    (parsed_config, output_dir)
}
