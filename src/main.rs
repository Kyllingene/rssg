mod build;
mod cache;
mod command;
mod error;
mod filepath;
mod filter;
mod parse;
mod rule;
mod tempfile;
mod template;

use std::fs::{self, File};
use std::io::ErrorKind;
use std::path::Path;
use std::process::exit;

use log::{error, info};
use sarge::prelude::*;

struct Args {
    help: bool,
    compile: bool,
    init: Option<String>,
    logfile: Option<String>,
    verbose: bool,
    force: bool,

    content: Option<String>,
    output: Option<String>,
    public: Option<String>,
    clean: bool,
}

fn main() {
    let parser = ArgumentParser::new();
    let args = {
        let help = parser.add(tag::both('h', "help"));
        let compile = parser.add(tag::both('c', "compile"));
        let init = parser.add(tag::both('i', "init"));
        let logfile = parser.add(tag::both('l', "logfile").env("RSSG_LOGFILE"));
        let verbose = parser.add(tag::both('v', "verbose"));
        let force = parser.add(tag::both('f', "force"));

        let content = parser.add(tag::long("content"));
        let output = parser.add(tag::long("output"));
        let public = parser.add(tag::long("public"));
        let clean = parser.add(tag::long("clean"));

        let _args = match parser.parse() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("ERROR: Failed to parse arguments: {e}");
                exit(1);
            }
        };

        Args {
            help: help.get().unwrap(),
            compile: compile.get().unwrap(),
            init: init.get().ok(),
            logfile: logfile.get().ok(),
            verbose: verbose.get().unwrap(),
            force: force.get().unwrap(),
            content: content.get().ok(),
            output: output.get().ok(),
            public: public.get().ok(),
            clean: clean.get().unwrap(),
        }
    };

    if args.help {
        println!(
            "{} [options]",
            parser.binary().unwrap_or_else(|| String::from("rssg"))
        );
        println!("     -h |    --help : print this help dialog");
        println!("     -c | --compile : compile the site");
        println!("     -i |    --init : create a new site");
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
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                record.level(),
                record.target(),
                message
            ))
        });

    if args.verbose {
        dis = dis.level(log::LevelFilter::Debug);
    } else {
        dis = dis.level(log::LevelFilter::Info);
    }

    if let Some(i) = args.logfile {
        let path = match fern::log_file(i) {
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

    let output = args.output.unwrap_or_else(|| String::from("output"));

    if let Some(new_dir) = args.init {
        if let Err(e) = std::fs::create_dir_all(&new_dir) {
            error!("Failed to create root directory: {e}");
            exit(1);
        }
        if let Err(e) = std::fs::create_dir_all(new_dir.clone() + "/content") {
            error!("Failed to create content directory: {e}");
            exit(1);
        }
        if let Err(e) = std::fs::create_dir_all(new_dir.clone() + "/public") {
            error!("Failed to create public directory: {e}");
            exit(1);
        }
        if let Err(e) = std::fs::create_dir_all(new_dir.clone() + "/templates") {
            error!("Failed to create templates directory: {e}");
            exit(1);
        }

        if let Err(e) = File::create(new_dir.clone() + "/rules.toml") {
            error!("Failed to create rules.toml: {e}");
            exit(1);
        }

        info!("Initialized new site at {new_dir}");
    }

    if args.clean {
        info!("Cleaning `{}` and `temp/`", output);
        if let Err(e) = fs::remove_dir_all(Path::new(&output)) {
            if e.kind() != ErrorKind::NotFound {
                error!("Failed to remove `{}`: {}", output, e);
                exit(1);
            }
        }

        if let Err(e) = fs::remove_dir_all(Path::new("temp")) {
            if e.kind() != ErrorKind::NotFound {
                error!("Failed to remove `temp/`: {}", e);
                exit(1);
            }
        }

        if let Err(e) = fs::remove_file(".rssg-cache") {
            if e.kind() != ErrorKind::NotFound {
                error!("Failed to remove `.rssg-cache`: {}", e);
                exit(1);
            }
        }
    }

    if args.compile {
        if !Path::new("rules.toml").exists() {
            error!("No `rules.toml` found, aborting");
            exit(1);
        }

        let content = args.content.unwrap_or_else(|| String::from("content"));

        let public = args.public.unwrap_or_else(|| String::from("public"));

        if !Path::new(&content).exists() {
            error!("Content directory (`{}`) not found, aborting", content);
            exit(1);
        }

        let data = match fs::read_to_string("rules.toml") {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to read `rules.toml`: {}", e);
                exit(1);
            }
        };

        let parse::ParsedDataResult { rules, pre_commands, post_commands } = match parse::parse(data) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to parse rules: {}", e);
                exit(1);
            }
        };

        if !build::build(
            &rules,
            pre_commands,
            post_commands,
            content,
            output,
            public,
            args.force,
        ) {
            error!("Build failed");
            exit(1);
        }
    }
}
