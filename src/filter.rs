use std::env;
use std::fs;
use std::fs::create_dir_all;
use std::{process::Command, str::FromStr};

use regex::Regex;

use crate::filepath::FilePath;
use crate::tempfile::tempdir;

fn program_in_path(program: String) -> bool {
    // TODO: windows uses ; as the path sep
    if let Ok(path) = env::var("PATH") {
        for p in path.split(":") {
            let p_str = format!("{}/{}", p, program);
            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

pub fn substitute(string: &String, path: &FilePath) -> String {
    string
        .replace("{full}", &path.full())
        .replace("{dir}", &path.dir())
        .replace("{name}", &path.name())
        .replace("{ext}", &path.ext())
        .replace("{\\{", "{{")
}

#[derive(Debug, Clone)]
pub struct Filter {
    command: String,
    outfile: String,
}

impl Filter {
    pub fn new(command: String, outfile: String) -> Self {
        Self { command, outfile }
    }

    pub fn tempdir(&self, path: &FilePath) -> Result<FilePath, String> {
        match FilePath::from_str(&substitute(&self.outfile, &path)) {
            Ok(new) => Ok(tempdir(&self.command, &new)),

            Err(e) => Err(format!(
                "[FAIL] Filter outfile {} invalid: {e}",
                self.outfile
            )),
        }
    }

    // Execute the filter.
    //
    // Logs directly to stdout/stderr. Returns true on a successful run.
    // Returns the frontmatter, if any.
    pub fn exec(&self, path: &FilePath) -> bool {
        // If outfile is an invalid path, then don't bother running the filter
        let out = match self.tempdir(&path) {
            Ok(new) => new,

            Err(e) => {
                eprintln!("{}", e);
                return false;
            }
        };

        if let Err(e) = create_dir_all(out.dir()) {
            eprintln!("[FAIL] Failed to create tempfile directory structure for filter: {e}");
            return false;
        }

        // Split command on non-quoted whitespace, removing the quotes
        let re = Regex::new("([^\"]+\"|[^\\s]+)").unwrap();
        let quotes = Regex::new("\"(.+)\"").unwrap();

        let subbed_command = substitute(&self.command, &path).replace("{outfile}", &out.full());

        let mut args = re.captures_iter(&subbed_command);

        let mut command = match args.next() {
            Some(c) => c,
            None => {
                eprintln!("[FAIL] `{}` is not a valid command", self.command);
                return false;
            }
        }[0]
        .to_string();

        command = quotes.replace_all(&command, "$1").to_string();

        if let Err(e) = create_dir_all(out.dir()) {
            eprintln!("[FAIL] Failed to create parent directories: {e}");
            return false;
        }

        println!("[INFO] Running filter `{subbed_command}`");
        if let Err(e) = Command::new(command)
            .args(
                args.map(|s| quotes.replace_all(&s[0].to_string(), "$1").to_string())
                    .collect::<Vec<_>>(),
            )
            .output()
        {
            eprintln!("[FAIL] Filter `{subbed_command}` failed with error: {e}");
            return false;
        }

        true
    }
}
