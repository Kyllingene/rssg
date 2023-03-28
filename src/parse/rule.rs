use std::collections::HashMap;

use crate::error::*;
use crate::filter::Filter;
use crate::parse::filter::parse_filter;
use crate::rule::Rule;
use crate::{array, field};

pub fn parse_rule(data: &toml::Table, filter_map: &HashMap<String, Filter>) -> ParseResult<Rule> {
    let rule = field!(data, rule, String);
    let fs = field!(data, filters, Array);
    let templates = array!(data, templates, String);
    let output = if let Some(toml::Value::String(o)) = &data.get("output") {
        Some(o.clone())
    } else {
        None
    };

    let mut filters = Vec::new();
    for filter in fs {
        if let toml::Value::String(f) = filter {
            if let Some(f) = filter_map.get(f) {
                filters.push(f.clone());
            } else {
                return Err(ParseError::NoSuchFilter(f.clone()));
            }
        } else if let toml::Value::Table(filter) = filter {
            filters.push(parse_filter(filter)?.0);
        }
    }

    let mut rule = Rule::new(rule, output).map_err(|_| ParseError::BadRegex(rule.clone()))?;
    rule.filter_all(filters);
    rule.template_all(templates.into_iter().cloned().collect());

    Ok(rule)
}

pub fn parse_rules(
    rules: &Vec<toml::Value>,
    filters: &HashMap<String, Filter>,
) -> ParseResult<Vec<Rule>> {
    let mut new = Vec::new();
    for i in rules {
        if let toml::Value::Table(v) = i {
            new.push(parse_rule(v, filters)?);
        } else {
            return Err(ParseError::BadArrayItem);
        }
    }

    Ok(new)
}
