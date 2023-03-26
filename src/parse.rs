pub mod common;
pub mod filter;
pub mod rule;

use crate::error::*;
use crate::field;
use crate::rule::Rule;

pub fn parse(data: String) -> ParseResult<Vec<Rule>> {
    let data: toml::Value = toml::from_str(&data).map_err(ParseError::TomlError)?;

    let filters = field!(data, filters, Array);
    let rules = field!(data, rules, Array);

    let filters = filter::parse_filters(filters)?;
    rule::parse_rules(rules, &filters)
}
