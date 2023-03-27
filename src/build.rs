use std::fs::{copy, create_dir_all, read_dir, remove_file, remove_dir, remove_dir_all, create_dir};
use std::io::ErrorKind;
use std::{fs, io, path::Path, str::FromStr};

use log::{info, error, debug, warn};

use crate::filepath::FilePath;
use crate::rule::Rule;
use crate::cache;

fn visit_dirs(dir: &Path) -> io::Result<Vec<FilePath>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in read_dir(dir)? {
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

pub fn build(rules: Vec<Rule>, content: String, output: String, public: String, force_recomp: bool) -> bool {
    if force_recomp {
        if let Err(e) = remove_file(".rssg-cache") {
            if !matches!(e.kind(), ErrorKind::NotFound) {
                error!("Failed to delete cache: {}", e);
                return false;
            }
        }
    }

    let content_files = visit_dirs(Path::new(&content))
        .unwrap();
    let public_files = visit_dirs(Path::new(&public))
        .unwrap();
    let template_files = visit_dirs(Path::new("templates"))
        .unwrap();

    let files = content_files.iter()
        .chain(public_files.iter())
        .chain(template_files.iter())
        .collect::<Vec<_>>();

    let mut file_cache = cache::read_cache(Path::new(".rssg-cache"));
    let modified = cache::modified(&file_cache, &content, &public, &String::from("templates"))
        .unwrap_or_default();

    let template_modified = modified.iter()
        .any(|s| s.dir().starts_with("templates/"));

    info!("Building site");
    info!("Removing outdated files");
    if file_cache.is_empty() {
        _ = remove_dir_all(&output);
        _ = remove_dir_all("temp");
    
        if let Err(e) = create_dir(Path::new(&output)) {
            error!("Failed to create `{}`: {}", output, e);
        }

        if let Err(e) = create_dir(Path::new("temp")) {
            error!("Failed to create `temp/`: {}", e);
        }
    } else {
        for file in file_cache.clone().keys() {
            if !files.contains(&file) {
                debug!("Outdated file `{}`", file.full());
                if file.full().starts_with(&public) {
                    let path = file.clone().strip_prefix(&public).prefix(&output);
                    if let Err(e) = remove_file(Path::new(&path.full())) {
                        warn!("Failed to delete outdated file `{}`: {}", path.full(), e);
                    } else {
                        info!("Deleted outdated file `{}`", path.full());
                    }

                    if let Err(e) = remove_dir(Path::new(&path.dir())) {
                        warn!("Failed to delete outdated directory `{}/`: {}", path.dir(), e);
                    } else {
                        info!("Deleted outdated directory `{}`", path.dir());
                    }

                    file_cache.remove(file);

                    continue;
                } else {
                    for rule in &rules {
                        if rule.matches(file) {
                            if let Ok(path) = rule.out(file) {
                                let path = path.strip_prefix(&content).prefix(&output);
                                if let Err(e) = remove_file(Path::new(&path.full())) {
                                    warn!("Failed to delete outdated file `{}`: {}", path.full(), e);
                                } else {
                                    info!("Deleted outdated file `{}`", path.full());
                                }

                                if let Err(e) = remove_dir(Path::new(&path.dir())) {
                                    warn!("Failed to delete outdated directory `{}/`: {}", path.dir(), e);
                                } else {
                                    info!("Deleted outdated directory `{}`", path.dir());
                                }
                            }

                            file_cache.remove(file);

                            continue;
                        }
                    }
                }
            }
        }
    }

    if let Err(e) = fs::create_dir_all(&output) {
        if !matches!(e.kind(), ErrorKind::AlreadyExists) {
            error!("Failed to make `{}/`: {}", output, e);
            return false;
        }
    }

    if let Err(e) = fs::create_dir_all("temp") {
        if !matches!(e.kind(), ErrorKind::AlreadyExists) {
            error!("Failed to make `temp/`: {}", e);
            return false;
        }
    }

    info!("Generating data from `{}/`", content);
    for file in &content_files {
        if !(template_modified || modified.contains(file)) {
            debug!("Skipping file `{}`", file.full());
            continue;
        }

        debug!("Caching file `{}`", file.full());
        cache::cache_file(Path::new(&file.full()), &mut file_cache);

        info!("Building file `{}`", file.full());
        for rule in &rules {
            if rule.matches(file) {
                debug!("Found matching rule");
                if !rule.exec(file.clone(), &content, &output) {
                    error!("Rule failed, aborting");
                    return false;
                }

                break;
            }
        }
    }

    info!("Site generation complete, copying `{}/`", public);

    for file in &public_files {
        if !modified.contains(file) {
            debug!("Skipping file `{}`", file.full());
            continue;
        }

        debug!("Caching file `{}`", file.full());
        cache::cache_file(Path::new(&file.full()), &mut file_cache);

        if let Err(e) = create_dir_all(file.clone().strip_prefix(&public).prefix(&output).dir()) {
            error!("Failed to create {}: {}", file.dir(), e);
            return false;
        }

        if let Err(e) = copy(
            file.full(),
            file.clone().strip_prefix(&public).prefix(&output).full(),
        ) {
            error!("Failed to copy {file}: {}", e);
            return false;
        }
    }

    for file in modified {
        debug!("Caching file `{}`", file.full());
        cache::cache_file(Path::new(&file.full()), &mut file_cache);
    }

    info!("Done building site, output at `{}/`", output);

    debug!("Writing cache");
    cache::write_cache(Path::new(".rssg-cache"), file_cache);

    true
}
