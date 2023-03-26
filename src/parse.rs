pub mod filter;
pub mod rule;
pub mod common;

use crate::error::*;
use crate::rule::Rule;
use crate::field;

pub fn parse(data: String) -> ParseResult<Vec<Rule>> {
    let data: toml::Value = toml::from_str(&data).map_err(ParseError::TomlError)?;

    let filters = field!(data, filters, Array);
    let rules = field!(data, rules, Array);

    let filters = filter::parse_filters(filters)?;
    rule::parse_rules(rules, &filters)
}