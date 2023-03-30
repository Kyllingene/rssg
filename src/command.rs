use std::process;

use fancy_regex::Regex;
use log::debug;
use serde::{de::Visitor, Deserialize};

use crate::filepath::FilePath;

pub fn substitute(string: &str, path: &FilePath) -> String {
    string
        .replace("{full}", &path.full())
        .replace("{dir}", &path.dir())
        .replace("{name}", &path.name())
        .replace("{ext}", &path.ext())
}

#[derive(Debug)]
pub enum ExitStatus {
    Success(String),

    InvalidCommand(String),

    NonZero(String, i32),
    Failed(String, std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Command {
    command: String,
}

impl Command {
    pub fn new(command: String) -> Self {
        Self { command }
    }

    pub fn str(&self) -> &str {
        &self.command
    }

    pub fn exec(&self, path: Option<&FilePath>, outfile: Option<FilePath>) -> ExitStatus {
        // Split command on non-quoted whitespace, removing the quotes
        let re = Regex::new("(\".*?(?<!\\\\)\"|[^ ])*").unwrap();
        let quotes = Regex::new("^\"(.*)\"$").unwrap();

        let subbed_command = if let Some(path) = path {
            if let Some(out) = outfile {
                substitute(&self.command, path).replace("{outfile}", &out.full())
            } else {
                substitute(&self.command, path)
            }
        } else {
            self.command.clone()
        }
        .replace("{\\{", "{{");

        let mut args = re.captures_iter(&subbed_command);
        let mut command = match args.next() {
            Some(Ok(c)) => c,
            Some(Err(_)) | None => {
                return ExitStatus::InvalidCommand(subbed_command);
            }
        }[0]
        .to_string();

        command = quotes.replace_all(&command, "$1").to_string();

        let args = args
            .map(|s| {
                quotes.replace_all(&s.unwrap()[0], "$1")
                    .replace("\\\"", "\"")
                    .to_string()
            })
            .collect::<Vec<_>>();

        debug!("Running command `{}` (full `{}`)", command, subbed_command);
        match process::Command::new(command).args(args).spawn() {
            Ok(mut child) => match child.wait() {
                Ok(code) => {
                    if !code.success() {
                        return ExitStatus::NonZero(subbed_command, code.code().unwrap_or(1));
                    }
                }

                Err(e) => {
                    return ExitStatus::Failed(subbed_command, e);
                }
            },

            Err(e) => {
                return ExitStatus::Failed(subbed_command, e);
            }
        }

        ExitStatus::Success(subbed_command)
    }
}

struct CommandVisitor;
impl<'de> Visitor<'de> for CommandVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string")
    }
}

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let command = deserializer.deserialize_string(CommandVisitor {})?;
        Ok(Command::new(command))
    }
}
