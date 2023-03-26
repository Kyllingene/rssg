use std::fs::copy;
use std::{fs, io, path::Path, str::FromStr};

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

    println!("[INFO] Building site");
    println!("[INFO] Generating data from `content/`");
    for file in files {
        println!("[INFO] Checking file {}", file.full());
        for rule in &rules {
            if rule.matches(&file) {
                println!("[INFO] Found matching rule");
                if !rule.exec(file) {
                    eprintln!("[FAIL] Rule failed, aborting");
                    return false;
                }

                break;
            }
        }
    }

    println!("[INFO] Site generation complete, copying `public/`");
    let files = visit_dirs(Path::new("public"))
        .unwrap()
        .iter()
        .map(|f| f.clone().strip_prefix("/"))
        .collect::<Vec<_>>();

    for file in files {
        if let Err(e) = copy(
            file.full(),
            file.clone().strip_prefix("public").prefix("output").full(),
        ) {
            eprintln!("[FAIL] Failed to copy {file}: {e}");
            return false;
        }
    }

    println!("[INFO] Done building site, output at `output/`");

    true
}
