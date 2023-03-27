use std::fs::{read_dir, read_to_string, OpenOptions, remove_file};
use std::io::Write;
use std::{io, path::Path, str::FromStr};
use std::collections::hash_map::{HashMap, DefaultHasher};
use std::hash::{Hash, Hasher};

use log::warn;

use crate::filepath::FilePath;

// Caching is lower-priority, don't stop anything if it fails

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

pub fn hash_file(file: &Path) -> Option<(FilePath, u64)> {
    let data = read_to_string(file).ok()?;

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let data = hasher.finish();

    let file = match FilePath::from_str(&file.display().to_string()) {
        Ok(f) => f,
        Err(_) => return None
    };

    Some((file, data))
}

pub fn cache_file(file: &Path, cache: &mut HashMap<FilePath, u64>) {
    let (file, data) = match hash_file(file) {
        Some(d) => d,
        None => {
            warn!("Failed to cache file `{}`", file.display());
            return;
        }
    };

    cache.insert(file, data);
}

pub fn write_cache(path: &Path, cache: HashMap<FilePath, u64>) {
    let mut cache_data = String::new();
    for (file, data) in cache {
        cache_data.push_str(format!("{file}  {data}\n").as_str());
    }

    match OpenOptions::new().write(true).truncate(true).create(true).open(path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(cache_data.as_bytes()) {
                warn!("Failed to write cache file at {}: {}", path.display(), e);
                if let Err(e) = remove_file(path) {
                    warn!("Failed to remove `.rssg-cache` for safety, please remove it manually then fix this: {}", e);
                }
            }
        }

        Err(e) => {
            warn!("Failed to open cache file at {}: {}", path.display(), e);
            if let Err(e) = remove_file(path) {
                warn!("Failed to remove `.rssg-cache` for safety, please remove it manually then fix this: {}", e);
            }
        }
    }
}

pub fn read_cache(path: &Path) -> HashMap<FilePath, u64> {
    let mut cache = HashMap::new();
    let cache_data = if let Ok(d) = read_to_string(path) {
        d
    } else {
        return cache;
    };

    let cache_data = cache_data.split_terminator('\n').filter(|s| !s.is_empty());

    for line in cache_data {
        let mut line = line.split_terminator("  ");

        let file = if let Some(Ok(f)) = line.next().map(FilePath::from_str) {
            f
        } else {
            warn!("Invalid entry in .rssg-cache");
            continue;
        };

        let data = if let Some(Ok(d)) = line.next().map(|s| s.parse::<u64>()) {
            d
        } else {
            warn!("Invalid hash in .rssg-cache");
            continue;
        };

        cache.insert(file, data);
    }

    cache
}

pub fn modified(cache: &HashMap<FilePath, u64>, content: &String, public: &String, templates: &String) -> Option<Vec<FilePath>> {
    let mut files = visit_dirs(Path::new(content))
        .unwrap();

    files.append(&mut visit_dirs(Path::new(public))
        .unwrap());

    files.append(&mut visit_dirs(Path::new(templates))
        .unwrap());

    let mut modified = Vec::with_capacity(files.len() / 2 + 1);
    for path in files {
        if let Some((file, data)) = hash_file(Path::new(&path.full())) {
            if cache.get(&file) != Some(&data) {
                modified.push(path);
            }
        }
    }

    Some(modified)
}