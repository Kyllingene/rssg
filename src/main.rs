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

use sarge::{ArgumentParser, arg, get_flag};

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'h', "help"));
    parser.add(arg!(flag, both, 'c', "compile"));
    // parser.add(arg!(string, both, 'l', "logfile"));

    let _args = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("[FAIL] Failed to parse arguments: {e}");
            exit(1);
        }
    };

    if get_flag!(parser, both, 'h', "help") {
        println!("{} [options]", parser.binary.unwrap_or_else(|| String::from("rssg")));
        println!("     -h |    --help : print this help dialog");
        println!("     -c | --compile : compile the site");

        return;
    }

    if get_flag!(parser, both, 'c', "compile") {
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
}
