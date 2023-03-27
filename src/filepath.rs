use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePath {
    dir: Option<String>,
    pub name: String,
    pub ext: String,
}

impl FilePath {
    pub fn new() -> Self {
        Self {
            dir: None,
            name: String::new(),
            ext: String::new(),
        }
    }

    pub fn strip_prefix<S: ToString>(mut self, pre: S) -> Self {
        let _pre = pre.to_string();
        let pre = _pre.as_str();
        let path = if let Some(dir) = &self.dir {
            Path::new(dir)
        } else {
            return self;
        };

        self.dir = if let Ok(nopre) = path.strip_prefix(pre) {
            Some(nopre.display().to_string())
        } else if let Ok(nopre) = path.strip_prefix("/") {
            if let Ok(nopre) = nopre.strip_prefix(pre) {
                Some(nopre.display().to_string())
            } else {
                self.dir
            }
        } else {
            self.dir
        };

        self
    }

    pub fn prefix<S: ToString>(mut self, pre: S) -> Self {
        let pre = pre.to_string();
        if let Some(path) = self.dir {
            self.dir = Some(format!(
                "{}/{}",
                pre.strip_suffix('/').map(String::from).unwrap_or(pre),
                path
            ));
        } else {
            self.dir = Some(pre.strip_suffix('/').map(String::from).unwrap_or(pre));
        }

        self
    }

    pub fn full(&self) -> String {
        self.to_string()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn dir(&self) -> String {
        self.dir.clone().unwrap_or_default()
    }

    pub fn ext(&self) -> String {
        self.ext.clone()
    }
}

impl Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            if let Some(dir) = self.dir.as_ref() {
                if dir.is_empty() {
                    String::new()
                } else {
                    format!("{dir}/")
                }
            } else {
                String::new()
            },
            self.name,
            if self.ext.is_empty() {
                String::new()
            } else {
                format!(".{}", self.ext)
            }
        )
    }
}

impl FromStr for FilePath {
    type Err = &'static str;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        let path = Path::new(path);
        let mut filepath = FilePath::new();

        filepath.dir = path.parent().map(|p| p.display().to_string());
        if let Some(s) = &filepath.dir {
            if s.is_empty() {
                filepath.dir = None;
            }
        }

        filepath.name = if let Some(name) = path.file_stem() {
            name.to_string_lossy().into_owned()
        } else {
            return Err("Must provide a filename (only provided a directory)");
        };

        filepath.ext = path
            .extension()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(String::new);

        Ok(filepath)
    }
}
