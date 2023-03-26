use std::collections::HashMap;

use crate::filter::Filter;
use crate::error::*;
use crate::field;

pub fn parse_filter(filter: &toml::Table) -> ParseResult<(Filter, Option<String>)> {
    let name = if let Some(toml::Value::String(s)) = &filter.get("name") {
        Some(s.clone())
    } else {
        None
    };

    let command = field!(filter, command, String).clone();
    let outfile = field!(filter, outfile, String).clone();

    Ok((Filter::new(command, outfile), name))
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