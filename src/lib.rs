pub mod args;
pub mod concatenate;
pub mod deduplicate;
pub mod split_sql;
pub mod block_writer;

use std::path::PathBuf;
use glob::glob;
use std::error::Error;
use std::io;

//
// Constants
//
pub const DEFAULT_PATTERN: &str = "./input/diff_result_*";
pub const DEFAULT_OUTPUT: &str = "./output";
pub const ALL_FILE_NAME: &str = "diff_result_ALL.txt";
pub const DEDUP_FILE_NAME: &str = "diff_deduplicated.txt";

pub const ALTER_PATTERN: &str = r"(?i)^ALTER\s+TABLE\s+[A-Za-z0-9_]+\.(T\d+)_[A-Za-z0-9_]+";
pub const CREATE_PATTERN: &str = r"(?i)^CREATE\s+TABLE\s+([A-Za-z0-9_]+)\.([A-Za-z0-9_]+)";
pub const GRANT_REF_PATTERN: &str = r"(?i)^GRANT\s+REFERENCES\s+ON\s+[A-Za-z0-9_]+\.[A-Za-z0-9_]+";

pub const ALTER_PREFIX: &str = "ALTER_";
pub const CREATE_PREFIX: &str = "CREATE_";
pub const STEP_FILE_FORMAT: &str = "STEP_{}_{}.sql";

pub const CREATE_TABLE_SUFFIX: &str = r#" TABLESPACE BO_D_RW_D_02
PARTITION BY RANGE (TECH_DUE_DT) INTERVAL(NUMTODSINTERVAL(1, 'DAY'))
(
  PARTITION BEFORE2022 VALUES LESS THAN (TO_DATE('2022-01-01','YYYY-MM-DD')),
  PARTITION BEFORE2023 VALUES LESS THAN (TO_DATE('2023-01-01','YYYY-MM-DD'))
)
COMPRESS FOR QUERY HIGH ROW LEVEL LOCKING"#;

pub const CREATE_INDEX_SUFFIX: &str = r#" TABLESPACE BO_I_RW_D_02
NOCOMPRESS LOCAL"#;

/// A block of SQL statements.
#[derive(Debug)]
pub struct SqlBlock {
    pub name: String,
    pub lines: Vec<String>,
}


/// Gather files matching a glob pattern.
pub fn gather_files(pattern_str: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    for entry in glob(pattern_str)? {
        match entry {
            Ok(path) => files.push(path),
            Err(e) => eprintln!("Warning: {}", e),
        }
    }
    if files.is_empty() {
        eprintln!("No files matched {:?}", pattern_str);
        std::process::exit(1);
    }
    Ok(files)
}

/// Convert a regex::Error into io::Error (used in `split_sql`).
pub fn to_io_error(e: regex::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}
