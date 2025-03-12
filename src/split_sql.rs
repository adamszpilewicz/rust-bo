use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use regex::Regex;

use crate::{
    SqlBlock,
    to_io_error,
    ALTER_PATTERN,
    CREATE_PATTERN,
    GRANT_REF_PATTERN,
    ALTER_PREFIX,
    CREATE_PREFIX,
};

/// Splits a deduplicated file into blocks of SQL statements based on CREATE/ALTER lines.
pub fn split_into_blocks(input_path: &Path) -> io::Result<Vec<SqlBlock>> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    let re_alter = Regex::new(ALTER_PATTERN).map_err(to_io_error)?;
    let re_create = Regex::new(CREATE_PATTERN).map_err(to_io_error)?;
    let re_grant_ref = Regex::new(GRANT_REF_PATTERN).map_err(to_io_error)?;

    let mut blocks = Vec::new();
    let mut current_block = SqlBlock {
        name: String::new(),
        lines: Vec::new(),
    };

    let total = lines.len();
    let mut i = 0;

    while i < total {
        let line = lines[i].trim();

        // 1) Try to handle ALTER
        if let Some(new_i) =
            handle_alter_line(line, i, &lines, &mut current_block, &mut blocks, &re_alter)
        {
            i = new_i;
            continue;
        }

        // 2) Try to handle CREATE
        if let Some(new_i) = handle_create_line(
            line,
            i,
            &lines,
            total,
            &mut current_block,
            &mut blocks,
            &re_create,
            &re_grant_ref,
            &re_alter,
        ) {
            i = new_i;
            continue;
        }

        // Otherwise skip
        i += 1;
    }

    finalize_block(&mut current_block, &mut blocks);
    Ok(blocks)
}

/// Handles an ALTER line, creating a new block if necessary.
fn handle_alter_line(
    line: &str,
    i: usize,
    lines: &[String],
    current_block: &mut SqlBlock,
    blocks: &mut Vec<SqlBlock>,
    re_alter: &Regex,
) -> Option<usize> {
    if let Some(caps) = re_alter.captures(line) {
        let t_number = caps[1].to_string();
        // finalize current block if there's one
        finalize_block(current_block, blocks);

        let block_name = format!("{}{}", ALTER_PREFIX, t_number);
        let single_block = SqlBlock {
            name: block_name,
            lines: vec![lines[i].clone()],
        };
        blocks.push(single_block);

        Some(i + 1)
    } else {
        None
    }
}

/// Handles a CREATE line, creating a new block if necessary.
fn handle_create_line(
    line: &str,
    mut i: usize,
    lines: &[String],
    total: usize,
    current_block: &mut SqlBlock,
    blocks: &mut Vec<SqlBlock>,
    re_create: &Regex,
    re_grant_ref: &Regex,
    re_alter: &Regex,
) -> Option<usize> {
    if let Some(caps) = re_create.captures(line) {
        finalize_block(current_block, blocks);

        // Possibly attach preceding GRANT REFERENCES
        if i > 0 {
            let prev_line = lines[i - 1].trim();
            if re_grant_ref.is_match(prev_line) {
                current_block.lines.push(lines[i - 1].clone());
            }
        }

        let raw_table_name = caps[2].to_string();
        let short_name = short_table_name(&raw_table_name);
        current_block.name = format!("{}{}", CREATE_PREFIX, short_name);

        // add the CREATE line
        current_block.lines.push(lines[i].clone());
        i += 1;

        // gather subsequent lines
        while i < total {
            let next_line = lines[i].trim();
            if next_line.is_empty() {
                i += 1;
                continue;
            }
            // break if we hit a new CREATE or ALTER line
            if re_create.is_match(next_line) || re_alter.is_match(next_line) {
                break;
            }
            current_block.lines.push(lines[i].clone());
            i += 1;
        }

        finalize_block(current_block, blocks);
        Some(i)
    } else {
        None
    }
}

/// Finalizes the current block and adds it to the list of blocks.
fn finalize_block(current_block: &mut SqlBlock, blocks: &mut Vec<SqlBlock>) {
    if !current_block.lines.is_empty() {
        let new_block = SqlBlock {
            name: current_block.name.clone(),
            lines: current_block.lines.clone(),
        };
        blocks.push(new_block);
        current_block.name.clear();
        current_block.lines.clear();
    }
}

/// Extract the portion of the table name before the first underscore
fn short_table_name(full: &str) -> String {
    if let Some(pos) = full.find('_') {
        full[..pos].to_string()
    } else {
        full.to_string()
    }
}
