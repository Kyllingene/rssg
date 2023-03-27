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

use log::error;
use sarge::{ArgumentParser, arg, get_flag, get_val};

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'h', "help"));
    parser.add(arg!(flag, both, 'c', "compile"));
    parser.add(arg!(str, both, 'l', "logfile"));
    parser.add(arg!(flag, both, 'v', "verbose"));

    let _args = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("ERROR: Failed to parse arguments: {}", e);
            exit(1);
        }
    };

    if get_flag!(parser, both, 'h', "help") {
        println!("{} [options]", parser.binary.unwrap_or_else(|| String::from("rssg")));
        println!("     -h |    --help : print this help dialog");
        println!("     -c | --compile : compile the site");
        println!("     -v | --verbose : include debug output");

        return;
    }

    let mut dis = fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                record.level(),
                record.target(),
                message
            ))
        });

    if get_flag!(parser, both, 'v', "verbose") {
        dis = dis.level(log::LevelFilter::Debug);
    } else {
        dis = dis.level(log::LevelFilter::Info);
    }

    if let Some(i) = get_val!(parser, both, 'l', "logfile") {
        let path = match fern::log_file(i.get_str()) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("ERROR: failed to establish logging to file: {e}");
                exit(1);
            }
        };

        if let Err(e) = dis.chain(path).apply() {
            eprintln!("ERROR: failed to establish logging to file: {e}");
            exit(1);
        }
    } else {
        if let Err(e) = dis.chain(std::io::stdout()).apply() {
            eprintln!("ERROR: failed to establish logging to stdout: {e}");
            exit(1);
        }
    }

    if get_flag!(parser, both, 'c', "compile") {
        if !Path::new("rules.toml").exists() {
            error!("No `rules.toml` found, aborting");
            exit(1);
        }

        if !Path::new("content").exists() {
            error!("No content directory found, aborting");
            exit(1);
        }

        let data = match read_to_string("rules.toml") {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to read `rules.toml`: {}", e);
                exit(1);
            }
        };

        let rules = match parse::parse(data) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to parse rules: {}", e);
                exit(1);
            }
        };

        if !build::build(rules) {
            error!("Build failed");
            exit(1);
        }
    }
}
