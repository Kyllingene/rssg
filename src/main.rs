mod build;
mod cache;
mod error;
mod filepath;
mod filter;
mod parse;
mod rule;
mod tempfile;
mod template;

use std::fs::{read_to_string, remove_dir_all, remove_file};
use std::io::ErrorKind;
use std::path::Path;
use std::process::exit;

use log::{error, info};
use sarge::{arg, get_flag, get_val, ArgumentParser};

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'h', "help"));
    parser.add(arg!(flag, both, 'c', "compile"));
    parser.add(arg!(str, both, 'l', "logfile"));
    parser.add(arg!(flag, both, 'v', "verbose"));
    parser.add(arg!(flag, both, 'f', "force"));

    parser.add(arg!(str, long, "content"));
    parser.add(arg!(str, long, "output"));
    parser.add(arg!(str, long, "public"));
    parser.add(arg!(flag, long, "clean"));

    let _args = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("ERROR: Failed to parse arguments: {e}");
            exit(1);
        }
    };

    if get_flag!(parser, both, 'h', "help") {
        println!(
            "{} [options]",
            parser.binary.unwrap_or_else(|| String::from("rssg"))
        );
        println!("     -h |    --help : print this help dialog");
        println!("     -c | --compile : compile the site");
        println!("     -v | --verbose : include debug output");
        println!("     -f |   --force : force recompilation");
        println!("                      rebuilds cache");
        println!("          --content : set source directory");
        println!("                      defaults to `content`");
        println!("           --output : set output directory");
        println!("                      defaults to `output`");
        println!("           --public : set public directory");
        println!("                      defaults to `public`");
        println!("            --clean : cleans the `output`");
        println!("                      and `temp` directories");

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
    } else if let Err(e) = dis.chain(std::io::stdout()).apply() {
        eprintln!("ERROR: failed to establish logging to stdout: {e}");
        exit(1);
    }

    let output = if let Some(c) = get_val!(parser, long, "output") {
        c.get_str()
    } else {
        String::from("output")
    };

    if get_flag!(parser, long, "clean") {
        info!("Cleaning `{}` and `temp/`", output);
        if let Err(e) = remove_dir_all(Path::new(&output)) {
            if e.kind() != ErrorKind::NotFound {
                error!("Failed to remove `{}`: {}", output, e);
                exit(1);
            }
        }

        if let Err(e) = remove_dir_all(Path::new("temp")) {
            if e.kind() != ErrorKind::NotFound {
                error!("Failed to remove `temp/`: {}", e);
                exit(1);
            }
        }

        if let Err(e) = remove_file(".rssg-cache") {
            if e.kind() != ErrorKind::NotFound {
                error!("Failed to remove `.rssg-cache`: {}", e);
                exit(1);
            }
        }
    }

    if get_flag!(parser, both, 'c', "compile") {
        if !Path::new("rules.toml").exists() {
            error!("No `rules.toml` found, aborting");
            exit(1);
        }

        let content = if let Some(c) = get_val!(parser, long, "content") {
            c.get_str()
        } else {
            String::from("content")
        };

        let public = if let Some(c) = get_val!(parser, long, "public") {
            c.get_str()
        } else {
            String::from("public")
        };

        if !Path::new(&content).exists() {
            error!("Content directory (`{}`) not found, aborting", content);
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

        if !build::build(
            rules,
            content,
            output,
            public,
            get_flag!(parser, both, 'f', "force"),
        ) {
            error!("Build failed");
            exit(1);
        }
    }
}
