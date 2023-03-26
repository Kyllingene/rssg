pub type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone)]
pub enum ParseError {
    // common
    MissingField(&'static str),
    BadArrayItem,
    TomlError(toml::de::Error),

    // Rule
    BadRegex(String),
    NoSuchFilter(String),

    // Filter
    MissingFilterName,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "Missing field `{field}`"),
            Self::BadArrayItem => write!(f, "Array item is incorrect type"),
            Self::TomlError(e) => write!(f, "An error occurred when parsing TOML: {e}"),

            Self::BadRegex(re) => write!(f, "The regex `{re}` is invalid"),
            Self::NoSuchFilter(filter) => write!(f, "The filter `{filter}` does not exist"),

            Self::MissingFilterName => write!(f, "Named filter is missing name"),
        }
    }
}

impl std::error::Error for ParseError {}
