use std::fs::create_dir_all;
use std::{process::Command, str::FromStr};

use log::{error, debug};
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
    outfile: Option<String>,
}

impl Filter {
    pub fn new(command: String, outfile: Option<String>) -> Self {
        Self { command, outfile }
    }

    pub fn tempdir(&self, path: &FilePath) -> Result<FilePath, String> {
        match FilePath::from_str(&substitute(self.outfile.as_ref().unwrap(), path)) {
            Ok(new) => Ok(tempdir(&self.command, &new)),

            Err(e) => Err(format!("Filter outfile {} invalid: {e}", self.outfile.as_ref().unwrap())),
        }
    }

    pub fn has_outfile(&self) -> bool {
        self.outfile.is_some()
    }

    // Execute the filter.
    //
    // Logs directly to stdout/stderr. Returns true on a successful run.
    // Returns the frontmatter, if any.
    pub fn exec(&self, path: &FilePath) -> bool {
        // If outfile is an invalid path, then don't bother running the filter
        let out = if self.has_outfile() {
            let out = match self.tempdir(path) {
                Ok(new) => new,

                Err(e) => {
                    error!("{}", e);
                    return false;
                }
            };

            if let Err(e) = create_dir_all(out.dir()) {
                error!("Failed to create tempfile directory structure for filter: {}", e);
                return false;
            }

            Some(out)
        } else {
            None
        };

        // Split command on non-quoted whitespace, removing the quotes
        let re = Regex::new("([^\"]+\"|[^\\s]+)").unwrap();
        let quotes = Regex::new("\"(.+)\"").unwrap();

        let subbed_command = substitute(&self.command, path).replace("{outfile}", &out.map(|f| f.full()).unwrap_or_else(|| String::from("null")));

        let mut args = re.captures_iter(&subbed_command);

        let mut command = match args.next() {
            Some(c) => c,
            None => {
                error!("`{}` is not a valid command", self.command);
                return false;
            }
        }[0]
        .to_string();

        command = quotes.replace_all(&command, "$1").to_string();

        debug!("Running filter `{}`", subbed_command);
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
                        error!(
                            "Filter `{}` exited with non-zero code: {}",
                            subbed_command,
                            code.code().unwrap_or(1)
                        );
                        return false;
                    }
                }

                Err(e) => {
                    error!("Filter `{}` failed with error: {}", subbed_command, e);
                    return false;
                }
            },

            Err(e) => {
                error!("Filter `{}` failed with error: {}", subbed_command, e);
                return false;
            }
        }

        debug!("Filter `{}` exited successfully", subbed_command);

        true
    }
}
