use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{digit1, multispace0, space1},
    combinator::{map, map_opt, map_parser},
    multi::many0,
    sequence::delimited,
    IResult, ParseTo,
};
use std::collections::HashMap;

use crate::helpers::*;

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Text(String),
    Int(i32),
}

impl PropertyValue {
    pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
        alt((property_value_string, property_value_int))(input)
    }
}

pub type Properties = HashMap<String, PropertyValue>;

fn property_value_string(input: &str) -> IResult<&str, PropertyValue> {
    map(
        map_opt(
            delimited(tag("\""), take_until("\""), tag("\"")),
            |s: &str| s.parse_to(),
        ),
        |s| PropertyValue::Text(s),
    )(input)
}

fn property_value_int(input: &str) -> IResult<&str, PropertyValue> {
    map(parse_to_i32, |i| PropertyValue::Int(i))(input)
}

fn property(input: &str) -> IResult<&str, (String, PropertyValue)> {
    let (input, _) = multispace0(input)?;
    let (input, key) = map_opt(take_until(" "), |s: &str| s.parse_to())(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = PropertyValue::parse(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, (key, value)))
}

fn num_properties(input: &str) -> IResult<&str, u32> {
    statement("STARTPROPERTIES", map_opt(digit1, |n: &str| n.parse_to()))(input)
}

pub fn parse_properties(input: &str) -> IResult<&str, Properties> {
    map(
        map_parser(
            delimited(
                num_properties,
                take_until("ENDPROPERTIES"),
                tag("ENDPROPERTIES"),
            ),
            many0(property),
        ),
        |res| res.iter().cloned().collect::<Properties>(),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn it_parses_whitespacey_properties() {
        assert_eq!(
            property("KEY   \"VALUE\""),
            Ok((
                "",
                ("KEY".to_string(), PropertyValue::Text("VALUE".to_string()))
            ))
        );

        assert_eq!(
            property("KEY   \"RANDOM WORDS AND STUFF\""),
            Ok((
                "",
                (
                    "KEY".to_string(),
                    PropertyValue::Text("RANDOM WORDS AND STUFF".to_string())
                )
            ))
        );
    }

    #[test]
    fn it_parses_string_properties() {
        assert_eq!(
            property("KEY \"VALUE\""),
            Ok((
                "",
                ("KEY".to_string(), PropertyValue::Text("VALUE".to_string()))
            ))
        );
    }

    #[test]
    fn it_parses_integer_properties() {
        assert_eq!(
            property("POSITIVE_NUMBER 10"),
            Ok((
                "",
                ("POSITIVE_NUMBER".to_string(), PropertyValue::Int(10i32))
            ))
        );

        assert_eq!(
            property("NEGATIVE_NUMBER -10"),
            Ok((
                "",
                ("NEGATIVE_NUMBER".to_string(), PropertyValue::Int(-10i32))
            ))
        );
    }

    #[test]
    fn it_parses_empty_properties() {
        let input = r#"STARTPROPERTIES 0
ENDPROPERTIES"#;

        assert_eq!(parse_properties(input), Ok(("", Properties::new())));
    }

    #[test]
    fn it_parses_properties() {
        let input = r#"STARTPROPERTIES 2
TEXT "FONT"
INTEGER 10
ENDPROPERTIES"#;

        let expected: Properties = hashmap![
            "TEXT".into() => PropertyValue::Text("FONT".into()),
            "INTEGER".into() => PropertyValue::Int(10),
        ];

        assert_eq!(parse_properties(input), Ok(("", expected)));
    }
}
