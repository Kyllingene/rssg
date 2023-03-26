use std::fs::{copy, create_dir_all, read_to_string, OpenOptions};
use std::io::Write;
use std::str::FromStr;

use regex::Regex;
use yaml_front_matter::YamlFrontMatter;

use crate::filepath::FilePath;
use crate::filter::{substitute, Filter};
use crate::tempfile::tempdir;
use crate::template::apply_template;

#[derive(Debug, Clone)]
pub struct Rule {
    rule: Regex,
    filters: Vec<Filter>,
    templates: Vec<String>,
    output: String,
}

impl Rule {
    pub fn new(rule: &str, output: String) -> Result<Self, regex::Error> {
        Ok(Self {
            rule: Regex::new(rule)?,
            filters: Vec::new(),
            templates: Vec::new(),
            output,
        })
    }

    pub fn filter_all(&mut self, mut filters: Vec<Filter>) {
        self.filters.append(&mut filters);
    }

    pub fn template_all(&mut self, mut templates: Vec<String>) {
        self.templates.append(&mut templates);
    }

    pub fn matches(&self, filepath: &FilePath) -> bool {
        self.rule.is_match(&filepath.full())
    }

    pub fn exec(&self, path: FilePath) -> bool {
        let data = match read_to_string(&path.full()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[FAIL:{}] Failed to open file {path}: {e}", line!());
                return false;
            }
        };

        let (yaml, data) = match YamlFrontMatter::parse::<serde_yaml::Value>(&data) {
            Ok(yaml) => (yaml.metadata, yaml.content),
            Err(_) => (serde_yaml::Value::Null, data),
        };

        let mut cwpath = tempdir(&format!("{path}-yamlless"), &path);

        if let Err(e) = create_dir_all(cwpath.dir()) {
            eprintln!(
                "[FAIL:{}] Failed to create parent directories: {e}",
                line!()
            );
            return false;
        }

        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&cwpath.full())
        {
            Ok(mut f) => {
                if let Err(e) = f.write_all(data.as_bytes()) {
                    eprintln!("[FAIL:{}] Failed to write to file {cwpath}: {e}", line!());
                    return false;
                }
            }
            Err(e) => {
                eprintln!("[FAIL:{}] Failed to open file {cwpath}: {e}", line!());
                return false;
            }
        }

        for filter in &self.filters {
            if !filter.exec(&cwpath) {
                return false;
            }

            cwpath = filter.tempdir(&cwpath).unwrap();
        }

        for template in &self.templates {
            println!("[INFO] Applying template file {template} to {cwpath}");

            let out = tempdir(template, &cwpath);

            if let Err(e) = create_dir_all(out.dir()) {
                eprintln!(
                    "[FAIL:{}] Failed to create parent directories: {e}",
                    line!()
                );
                return false;
            }

            if let Err(e) = create_dir_all(out.dir()) {
                eprintln!(
                    "[FAIL:{}] Failed to create tempfile directory structure for template: {e}",
                    line!()
                );
                return false;
            }

            if let Err(e) = apply_template(template, &cwpath.full(), out.full(), &yaml) {
                eprintln!("[FAIL:{}] Failed to apply template: {e}", line!());
                return false;
            }

            cwpath = tempdir(template, &cwpath);
        }

        let out = match FilePath::from_str(&substitute(&self.output, &path)) {
            Ok(f) => f.strip_prefix("content").prefix("output"),
            Err(e) => {
                eprintln!("[FAIL:{}] Failed to create final file: {e}", line!());
                return false;
            }
        };

        if let Err(e) = create_dir_all(out.dir()) {
            eprintln!(
                "[FAIL:{}] Failed to create final file parent directories: {e}",
                line!()
            );
            return false;
        }

        if let Err(e) = copy(cwpath.full(), out.full()) {
            eprintln!("[FAIL:{}] Failed to finalize file output: {e}", line!());
            return false;
        }

        true
    }
}
