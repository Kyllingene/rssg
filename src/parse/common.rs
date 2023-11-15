#[macro_export]
macro_rules! field {
    ( $root:ident, $name:ident, $typ:ident ) => {
        if let Some(toml::Value::$typ(v)) = &$root.get(stringify!($name)) {
            v
        } else {
            return Err(ParseError::MissingField(stringify!($name)));
        }
    };
}

#[macro_export]
macro_rules! array {
    ( $d:ident, $n:ident, $t:ident ) => {{
        let mut new = Vec::new();
        for i in field!($d, $n, Array) {
            if let toml::Value::$t(v) = i {
                new.push(v);
            } else {
                return Err(ParseError::BadArrayItem);
            }
        }

        new
    }};
}
