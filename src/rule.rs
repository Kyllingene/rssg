use std::fs::{copy, create_dir_all, read_to_string, OpenOptions};
use std::io::Write;
use std::str::FromStr;

use log::{info, error};
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
                error!("Failed to open file {}: {}", path, e);
                return false;
            }
        };

        let (yaml, data) = match YamlFrontMatter::parse::<serde_yaml::Value>(&data) {
            Ok(yaml) => (yaml.metadata, yaml.content),
            Err(_) => (serde_yaml::Value::Null, data),
        };

        let mut cwpath = tempdir(&format!("{path}-yamlless"), &path);

        if let Err(e) = create_dir_all(cwpath.dir()) {
            error!(
                "Failed to create parent directories: {}",
                e
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
                    error!("Failed to write to file {}: {}", cwpath, e);
                    return false;
                }
            }
            Err(e) => {
                error!("Failed to open file {}: {}", cwpath, e);
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
            info!("Applying template file {} to {}", template, cwpath);

            let out = tempdir(template, &cwpath);

            if let Err(e) = create_dir_all(out.dir()) {
                error!(
                    "Failed to create parent directories: {}",
                    e
                );
                return false;
            }

            if let Err(e) = create_dir_all(out.dir()) {
                error!(
                    "Failed to create tempfile directory structure for template: {}",
                    e
                );
                return false;
            }

            if let Err(e) = apply_template(template, &cwpath.full(), out.full(), &yaml) {
                error!("Failed to apply template: {}", e);
                return false;
            }

            cwpath = tempdir(template, &cwpath);
        }

        let out = match FilePath::from_str(&substitute(&self.output, &path)) {
            Ok(f) => f.strip_prefix("content").prefix("output"),
            Err(e) => {
                error!("Failed to create final file: {}", e);
                return false;
            }
        };

        if let Err(e) = create_dir_all(out.dir()) {
            error!(
                "Failed to create final file parent directories: {}", e
            );
            return false;
        }

        if let Err(e) = copy(cwpath.full(), out.full()) {
            error!("Failed to finalize file output: {}", e);
            return false;
        }

        true
    }
}
