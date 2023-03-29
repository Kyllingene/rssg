pub mod common;
pub mod filter;
pub mod rule;

use crate::command::Command;
use crate::error::*;
use crate::field;
use crate::rule::Rule;

type ParsedDataResult = ParseResult<(Vec<Rule>, (Vec<Command>, Vec<Command>))>;

pub fn parse(data: String) -> ParsedDataResult {
    let data: toml::Value = toml::from_str(&data).map_err(ParseError::TomlError)?;

    let filters = field!(data, filters, Array);
    let rules = field!(data, rules, Array);

    let pre_commands = if let Some(toml::Value::Array(commands)) = &data.get("pre_commands") {
        commands
            .iter()
            .filter_map(|v| v.as_str().map(|command| Command::new(command.to_string())))
            .collect()
    } else {
        Vec::new()
    };

    let post_commands = if let Some(toml::Value::Array(commands)) = &data.get("post_commands") {
        commands
            .iter()
            .filter_map(|v| v.as_str().map(|command| Command::new(command.to_string())))
            .collect()
    } else {
        Vec::new()
    };

    let filters = filter::parse_filters(filters)?;
    let rules = rule::parse_rules(rules, &filters)?;

    Ok((rules, (pre_commands, post_commands)))
}
