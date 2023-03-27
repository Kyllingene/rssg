use std::fs::{copy, create_dir_all};
use std::{fs, io, path::Path, str::FromStr};

use log::{info, error, debug};

use crate::filepath::FilePath;
use crate::rule::Rule;

fn visit_dirs(dir: &Path) -> io::Result<Vec<FilePath>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.append(&mut visit_dirs(&path)?);
            } else {
                files.push(FilePath::from_str(&entry.path().to_string_lossy()).unwrap());
            }
        }
    }

    Ok(files)
}

pub fn build(rules: Vec<Rule>) -> bool {
    _ = fs::remove_dir_all("output");
    _ = fs::remove_dir_all("temp");

    fs::create_dir_all("output").unwrap();
    fs::create_dir_all("temp").unwrap();

    let files = visit_dirs(Path::new("content"))
        .unwrap()
        .iter()
        .map(|f| f.clone().strip_prefix("/"))
        .collect::<Vec<_>>();

    info!("Building site");
    info!("Generating data from `content/`");
    for file in files {
        info!("Building file {}", file.full());
        for rule in &rules {
            if rule.matches(&file) {
                debug!("Found matching rule");
                if !rule.exec(file) {
                    error!("Rule failed, aborting");
                    return false;
                }

                break;
            }
        }
    }

    info!("Site generation complete, copying `public/`");
    let files = visit_dirs(Path::new("public"))
        .unwrap()
        .iter()
        .map(|f| f.clone().strip_prefix("/"))
        .collect::<Vec<_>>();

    for file in files {
        if let Err(e) = create_dir_all(file.clone().strip_prefix("public").prefix("output").dir()) {
            error!("Failed to create {}: {}", file.dir(), e);
            return false;
        }

        if let Err(e) = copy(
            file.full(),
            file.clone().strip_prefix("public").prefix("output").full(),
        ) {
            error!("Failed to copy {file}: {}", e);
            return false;
        }
    }

    info!("Done building site, output at `output/`");

    true
}
