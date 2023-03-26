use std::error::Error;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use serde_yaml::Value;

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum TemplateErr {
    TemplateNotFound(String),
    FileNotFound(String),

    FailedToReadTemplate(std::io::Error),
    FailedToReadFile(std::io::Error),

    FailedToWrite(std::io::Error),
}

impl Display for TemplateErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TemplateNotFound(file) => write!(f, "Failed to find template file {file}"),
            Self::FileNotFound(file) => write!(f, "Failed to find input file {file}"),

            Self::FailedToReadTemplate(e) => write!(f, "Failed to read template file: {e}"),
            Self::FailedToReadFile(e) => write!(f, "Failed to read input file: {e}"),

            Self::FailedToWrite(e) => write!(f, "Failed to write output file: {e}"),
        }
    }
}

impl Error for TemplateErr {}

pub fn apply_template<T, F, O>(template: T, file: F, out: O, yaml: &Value) -> Result<(), TemplateErr>
where
    T: AsRef<Path>,
    F: AsRef<Path>,
    O: AsRef<Path>,
{
    if !template.as_ref().exists() {
        return Err(TemplateErr::TemplateNotFound(
            template.as_ref().display().to_string(),
        ));
    }

    if !file.as_ref().exists() {
        return Err(TemplateErr::FileNotFound(
            file.as_ref().display().to_string(),
        ));
    }

    let mut template_data = match File::open(&template) {
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

    template_data = template_data
        .replace("{{data}}", data.as_str())
        .replace("{{version}}", &format!("{}", VERSION.unwrap_or("unknown")));

    let mut last = 0;
    while let Some(mut start) = template_data[(last as usize)..].find("{{data.") {
        start += last as usize;
        let temp = &template_data[start..];
        let end = match temp.find("}}") {
            Some(i) => i+start,
            None => continue, 
        };
        
        let ident = &template_data[start+7..end];
        let length = if let Some(val) = yaml.get(ident) {
            let to_rep = format!("{{{{data.{ident}}}}}");
            let rep_with = match val {
                Value::String(s) => s.clone(),
                Value::Bool(v) => format!("{v}"),
                Value::Number(v) => format!("{v}"),
                Value::Null => "null".to_string(),
                _ => format!("{val:?}")
            };
            
            template_data = template_data.replace(&to_rep, &rep_with);

            rep_with.len() as i64 - to_rep.len() as i64
        } else {
            0
        };

        last = end as i64 + length;
    }

    template_data = template_data.replace("{\\{", "{{");

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
