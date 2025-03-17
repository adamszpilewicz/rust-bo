use std::fs;
use std::path::PathBuf;
use std::env;
use serde::Deserialize;
use serde_yaml;

// If you have these constants
use crate::{DEFAULT_PATTERN, DEFAULT_OUTPUT};

#[derive(Debug, Deserialize)]
pub struct Config {
    /// The pattern for input files; fallback to DEFAULT_PATTERN if not given
    pub pattern: Option<String>,
    /// The path to the mod_file; fallback to none or something else if not given
    pub mod_file: Option<String>,
    /// The output directory; fallback to DEFAULT_OUTPUT if not given
    pub output: Option<String>,
}

/// parse_args: load config from "config.yaml", apply defaults, and
/// still allow optional overrides from `--pattern=` or `--output=` on CLI.
///
/// Returns (pattern_string, mod_file_string, output_directory_pathbuf)
pub fn parse_args() -> (String, String, PathBuf) {
    let config_bytes = fs::read("config.yaml")
        .expect("Failed to read config.yaml. Please ensure it exists.");

    let parsed_config: Config = serde_yaml::from_slice(&config_bytes)
        .expect("Invalid YAML in config.yaml.");

    // Apply default for pattern if missing
    let mut pattern_str = parsed_config
        .pattern
        .unwrap_or_else(|| DEFAULT_PATTERN.to_string());

    // We won't have a default for mod_file, let's keep it empty if not specified
    let mod_file_str = parsed_config
        .mod_file
        .unwrap_or_else(|| "".to_string());

    // Apply default for output if missing
    let mut output_str = parsed_config
        .output
        .unwrap_or_else(|| DEFAULT_OUTPUT.to_string());

    // Optional: override some with CLI flags
    for arg in env::args().skip(1) {
        if let Some(stripped) = arg.strip_prefix("--pattern=") {
            pattern_str = stripped.to_string();
        } else if let Some(stripped) = arg.strip_prefix("--output=") {
            output_str = stripped.to_string();
        }
        // If you want a CLI override for mod_file, add the code:
        // else if let Some(stripped) = arg.strip_prefix("--mod_file=") {
        //     mod_file_str = stripped.to_string();
        // }
    }

    let output_dir = PathBuf::from(&output_str);
    (pattern_str, mod_file_str, output_dir)
}
