use std::collections::HashMap;

use crate::error::*;
use crate::field;
use crate::filter::Filter;

pub fn parse_filter(filter: &toml::Table) -> ParseResult<(Filter, Option<String>)> {
    let name = if let Some(toml::Value::String(s)) = &filter.get("name") {
        Some(s.clone())
    } else {
        None
    };

    let command = field!(filter, command, String).clone();
    let outfile = if let Some(toml::Value::String(o)) = &filter.get("outfile") {
        Some(o.clone())
    } else {
        None
    };

    let give_original = matches!(&filter.get("give_original"), Some(toml::Value::Boolean(true)))
        && outfile.is_none();

    Ok((Filter::new(command, outfile, give_original), name))
}

pub fn parse_filters(filters: &Vec<toml::Value>) -> ParseResult<HashMap<String, Filter>> {
    let mut new = HashMap::new();
    for i in filters {
        if let toml::Value::Table(v) = i {
            let (filter, name) = parse_filter(v)?;
            new.insert(name.ok_or(ParseError::MissingFilterName)?, filter);
        } else {
            return Err(ParseError::BadArrayItem);
        }
    }

    Ok(new)
}
