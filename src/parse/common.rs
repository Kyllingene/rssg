#[macro_export]
macro_rules! field {
    ( $d:ident, $n:ident, $t:ident ) => {
        if let Some(toml::Value::$t(v)) = &$d.get(stringify!($n)) {
            v
        } else {
            return Err(ParseError::MissingField(stringify!($n)));
        }
    };
}

#[macro_export]
macro_rules! array {
    ( $d:ident, $n:ident, $t:ident ) => {
        {
            let mut new = Vec::new();
            for i in field!($d, $n, Array) {
                if let toml::Value::$t(v) = i {
                    new.push(v);
                } else {
                    return Err(ParseError::BadArrayItem);
                }
            }

            new
        }
    };
}