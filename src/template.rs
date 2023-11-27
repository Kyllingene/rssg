use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;

use handlebars::{Handlebars, RenderError};
use serde_yaml::{Mapping, Value};

use crate::filepath::FilePath;

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum TemplateErr {
    InvalidPath(String),

    TemplateNotFound(String),
    InvalidTemplate(RenderError),
    FileNotFound(String),

    FailedToReadTemplate(std::io::Error),
    FailedToReadFile(std::io::Error),

    FailedToWrite(std::io::Error),
}

impl Display for TemplateErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath(file) => write!(f, "Invalid file path {file}"),

            Self::TemplateNotFound(file) => write!(f, "Failed to find template file {file}"),
            Self::InvalidTemplate(e) => write!(f, "Failed to apply template: {e}"),
            Self::FileNotFound(file) => write!(f, "Failed to find input file {file}"),

            Self::FailedToReadTemplate(e) => write!(f, "Failed to read template file: {e}"),
            Self::FailedToReadFile(e) => write!(f, "Failed to read input file: {e}"),

            Self::FailedToWrite(e) => write!(f, "Failed to write output file: {e}"),
        }
    }
}

impl Error for TemplateErr {}

pub fn apply_template<T, F, O>(
    template: T,
    file: F,
    out: O,
    yaml: &Mapping,
) -> Result<(), TemplateErr>
where
    T: AsRef<Path>,
    F: AsRef<Path>,
    O: AsRef<Path>,
{
    let t = FilePath::from_str(&template.as_ref().display().to_string())
        .map_err(|_| TemplateErr::InvalidPath(template.as_ref().display().to_string()))?
        .prefix("templates")
        .to_string();

    let template = Path::new(&t);

    if !template.exists() {
        return Err(TemplateErr::TemplateNotFound(
            template.display().to_string(),
        ));
    }

    if !file.as_ref().exists() {
        return Err(TemplateErr::FileNotFound(
            file.as_ref().display().to_string(),
        ));
    }

    let mut template_data = match File::open(template) {
        Ok(mut f) => {
            let mut buf = String::new();

            if let Err(e) = f.read_to_string(&mut buf) {
                return Err(TemplateErr::FailedToReadTemplate(e));
            }

            buf
        }

        Err(e) => return Err(TemplateErr::FailedToReadTemplate(e)),
    };

    let data = match File::open(&file) {
        Ok(mut f) => {
            let mut buf = String::new();

            if let Err(e) = f.read_to_string(&mut buf) {
                return Err(TemplateErr::FailedToReadFile(e));
            }

            buf
        }

        Err(e) => return Err(TemplateErr::FailedToReadFile(e)),
    };

    template_data = template_data.replace("{{data}}", &data);
    template_data = template_data.replace("{{ data }}", &data);

    let mut vars: HashMap<Value, Value> = HashMap::from_iter(yaml.clone());
    vars.insert("version".into(), VERSION.unwrap_or("unknown").into());

    let reg = Handlebars::new();
    template_data = reg
        .render_template(&template_data, &vars)
        .map_err(TemplateErr::InvalidTemplate)?;

    match OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&out)
    {
        Ok(mut f) => {
            if let Err(e) = f.write_all(template_data.as_bytes()) {
                return Err(TemplateErr::FailedToWrite(e));
            }

            Ok(())
        }

        Err(e) => Err(TemplateErr::FailedToWrite(e)),
    }
}
