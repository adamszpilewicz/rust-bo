use std::env;
use std::path::PathBuf;

use crate::{DEFAULT_PATTERN, DEFAULT_OUTPUT};

/// parse_args: read command-line flags:
///   --pattern=<someglob>  (defaults to DEFAULT_PATTERN)
///   --output=<somepath>   (defaults to DEFAULT_OUTPUT)
/// Returns (pattern_string, output_directory_pathbuf)
pub fn parse_args() -> (String, PathBuf) {
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
