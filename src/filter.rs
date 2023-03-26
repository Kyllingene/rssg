use std::fs::create_dir_all;
use std::{process::Command, str::FromStr};

use regex::Regex;
use serde::Deserialize;

use crate::filepath::FilePath;
use crate::tempfile::tempdir;

pub fn substitute(string: &str, path: &FilePath) -> String {
    string
        .replace("{full}", &path.full())
        .replace("{dir}", &path.dir())
        .replace("{name}", &path.name())
        .replace("{ext}", &path.ext())
        .replace("{\\{", "{{")
}

#[derive(Debug, Clone, Deserialize)]
pub struct Filter {
    command: String,
    outfile: String,
}

impl Filter {
    pub fn new(command: String, outfile: String) -> Self {
        Self { command, outfile }
    }

    pub fn tempdir(&self, path: &FilePath) -> Result<FilePath, String> {
        match FilePath::from_str(&substitute(&self.outfile, path)) {
            Ok(new) => Ok(tempdir(&self.command, &new)),

            Err(e) => Err(format!("Filter outfile {} invalid: {e}", self.outfile)),
        }
    }

    // Execute the filter.
    //
    // Logs directly to stdout/stderr. Returns true on a successful run.
    // Returns the frontmatter, if any.
    pub fn exec(&self, path: &FilePath) -> bool {
        // If outfile is an invalid path, then don't bother running the filter
        let out = match self.tempdir(path) {
            Ok(new) => new,

            Err(e) => {
                eprintln!("[FAIL] {e}");
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

        let subbed_command = substitute(&self.command, path).replace("{outfile}", &out.full());

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
        match Command::new(command)
            .args(
                args.map(|s| quotes.replace_all(&s[0], "$1").to_string())
                    .collect::<Vec<_>>(),
            )
            .spawn()
        {
            Ok(mut child) => match child.wait() {
                Ok(code) => {
                    if !code.success() {
                        eprintln!(
                            "[FAIL] Filter `{subbed_command}` exited with non-zero code: {}",
                            code.code().unwrap_or(1)
                        );
                        return false;
                    }
                }

                Err(e) => {
                    eprintln!("[FAIL] Filter `{subbed_command}` failed with error: {e}");
                    return false;
                }
            },

            Err(e) => {
                eprintln!("[FAIL] Filter `{subbed_command}` failed with error: {e}");
                return false;
            }
        }

        println!("[INFO] Filter `{subbed_command}` exited successfully");

        true
    }
}
