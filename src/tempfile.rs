use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::filepath::FilePath;

// example:
//   source: `default.html`
//     file: `content/home/index.html`
//   output: `temp/8264099790966940254/index.html`
pub fn tempdir(source: &str, filepath: &FilePath) -> FilePath {
    let mut s = DefaultHasher::new();
    source.hash(&mut s);
    filepath.full().hash(&mut s);

    let mut new = FilePath::new();
    new.name = filepath.name();
    new.ext = filepath.ext();
    new.prefix(s.finish()).prefix("temp")
}
