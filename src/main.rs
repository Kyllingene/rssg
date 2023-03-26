mod build;
mod error;
mod filepath;
mod filter;
mod parse;
mod rule;
mod tempfile;
mod template;

use std::fs::read_to_string;
use std::path::Path;
use std::process::exit;

fn main() {
    if !Path::new("rules.toml").exists() {
        eprintln!("[FAIL] No `rules.toml` found, aborting");
        exit(1);
    }

    if !Path::new("content").exists() {
        eprintln!("[FAIL] No content directory found, aborting");
        exit(1);
    }

    let data = match read_to_string("rules.toml") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[FAIL] Failed to read `rules.toml`: {e}");
            exit(1);
        }
    };

    let rules = match parse::parse(data) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[FAIL] Failed to parse rules: {e}");
            exit(1);
        }
    };

    if !build::build(rules) {
        eprintln!("[FAIL] Build failed");
        exit(1);
    }
}
