use std::fs::create_dir_all;
use std::str::FromStr;

use log::{debug, error};
use serde::Deserialize;

use crate::command::{Command, ExitStatus};
use crate::filepath::FilePath;
use crate::tempfile::tempdir;

pub fn substitute(string: &str, path: &FilePath) -> String {
    string
        .replace("{full}", &path.full())
        .replace("{dir}", &path.dir())
        .replace("{name}", &path.name())
        .replace("{ext}", &path.ext())
}

#[derive(Debug, Clone, Deserialize)]
pub struct Filter {
    command: Command,
    outfile: Option<String>,
    pub give_original: bool,
}

impl Filter {
    pub fn new(command: String, outfile: Option<String>, give_original: bool) -> Self {
        Self {
            command: Command::new(command),
            outfile,
            give_original,
        }
    }

    pub fn tempdir(&self, path: &FilePath) -> Result<FilePath, String> {
        match FilePath::from_str(
            &substitute(self.outfile.as_ref().unwrap(), path).replace("{\\{", "{{"),
        ) {
            Ok(new) => Ok(tempdir(self.command.str(), &new)),

            Err(e) => Err(format!(
                "Filter outfile {} invalid: {e}",
                self.outfile.as_ref().unwrap()
            )),
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
                error!(
                    "Failed to create tempfile directory structure for filter: {}",
                    e
                );
                return false;
            }

            Some(out)
        } else {
            None
        };

        debug!("Running filter `{}`", self.command.str());
        match self.command.exec(Some(path), out) {
            ExitStatus::Success(cmd) => {
                debug!("Filter `{}` exited successfully", cmd);
                true
            }

            ExitStatus::InvalidCommand(cmd) => {
                error!("Filter `{}` failed: not a valid command", cmd);
                false
            }
            ExitStatus::NonZero(cmd, code) => {
                error!(
                    "Filter `{}` failed: exited with non-zero code {}",
                    cmd, code
                );
                false
            }
            ExitStatus::Failed(cmd, e) => {
                error!("Filter `{}` failed: {}", cmd, e);
                false
            }
        }
    }
}
