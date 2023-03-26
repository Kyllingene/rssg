#![allow(dead_code)]

mod filepath;
mod filter;
mod rule;
mod tempfile;
mod template;

use std::{fs, io, path::Path, process::exit, str::FromStr};

fn visit_dirs(dir: &Path) -> io::Result<Vec<filepath::FilePath>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.append(&mut visit_dirs(&path)?);
            } else {
                files.push(
                    filepath::FilePath::from_str(&entry.path().to_string_lossy().to_string())
                        .unwrap(),
                );
            }
        }
    }

    Ok(files)
}

fn main() {
    let mut rules: Vec<rule::Rule> = Vec::new();

    let mut adoc =
        rule::Rule::new(".*index\\.(adoc|asciidoc)", "{dir}/index.html".to_string()).unwrap();
    adoc.template("templates/default.html".to_string());

    let adoc_f = filter::Filter::new(
        "asciidoctor --no-header-footer {full} -o {outfile}".to_string(),
        "{dir}/index.html".to_string(),
    );

    adoc.filter(adoc_f);
    rules.push(adoc);

    let mut adoc = rule::Rule::new(
        ".*\\.(adoc|asciidoc)",
        "{dir}/{name}/index.html".to_string(),
    )
    .unwrap();
    adoc.template("templates/default.html".to_string());

    let adoc_f = filter::Filter::new(
        "asciidoctor --no-header-footer {full} -o {outfile}".to_string(),
        "{dir}/{name}/index.html".to_string(),
    );

    adoc.filter(adoc_f);
    rules.push(adoc);

    let html = rule::Rule::new(".*index\\.html", "{dir}/index.html".to_string()).unwrap();
    rules.push(html);

    let html = rule::Rule::new(".*\\.html", "{dir}/{name}/index.html".to_string()).unwrap();
    rules.push(html);

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
    for file in files {
        println!("[INFO] Checking file {}", file.full());
        for rule in &rules {
            if rule.matches(&file) {
                println!("[INFO] Found matching rule");
                if !rule.exec(file) {
                    eprintln!("[FAIL] Rule failed, aborting");
                    exit(1);
                }

                break;
            }
        }
    }

    println!("[INFO] Done building site, output at `output/`");
}
