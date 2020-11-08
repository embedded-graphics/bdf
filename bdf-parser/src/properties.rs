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

pub type Properties = HashMap<String, PropertyValue>;

fn property_value_string(input: &[u8]) -> IResult<&[u8], PropertyValue> {
    map(
        map_opt(
            delimited(tag("\""), take_until("\""), tag("\"")),
            |s: &[u8]| s.parse_to(),
        ),
        |s| PropertyValue::Text(s),
    )(input)
}

fn property_value_int(input: &[u8]) -> IResult<&[u8], PropertyValue> {
    map(parse_to_i32, |i| PropertyValue::Int(i))(input)
}

fn property_value(input: &[u8]) -> IResult<&[u8], PropertyValue> {
    alt((property_value_string, property_value_int))(input)
}

fn property(input: &[u8]) -> IResult<&[u8], (String, PropertyValue)> {
    let (input, _) = multispace0(input)?;
    let (input, key) = map_opt(take_until(" "), |s: &[u8]| s.parse_to())(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = property_value(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, (key, value)))
}

fn num_properties(input: &[u8]) -> IResult<&[u8], u32> {
    statement("STARTPROPERTIES", map_opt(digit1, |n: &[u8]| n.parse_to()))(input)
}

pub fn properties(input: &[u8]) -> IResult<&[u8], Properties> {
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

    const EMPTY: &[u8] = &[];

    #[test]
    fn it_parses_whitespacey_properties() {
        assert_eq!(
            property(b"KEY   \"VALUE\""),
            Ok((
                EMPTY,
                ("KEY".to_string(), PropertyValue::Text("VALUE".to_string()))
            ))
        );

        assert_eq!(
            property(b"KEY   \"RANDOM WORDS AND STUFF\""),
            Ok((
                EMPTY,
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
            property(b"KEY \"VALUE\""),
            Ok((
                EMPTY,
                ("KEY".to_string(), PropertyValue::Text("VALUE".to_string()))
            ))
        );
    }

    #[test]
    fn it_parses_integer_properties() {
        assert_eq!(
            property(b"POSITIVE_NUMBER 10"),
            Ok((
                EMPTY,
                ("POSITIVE_NUMBER".to_string(), PropertyValue::Int(10i32))
            ))
        );

        assert_eq!(
            property(b"NEGATIVE_NUMBER -10"),
            Ok((
                EMPTY,
                ("NEGATIVE_NUMBER".to_string(), PropertyValue::Int(-10i32))
            ))
        );
    }

    #[test]
    fn it_parses_empty_properties() {
        let input = r#"STARTPROPERTIES 0
ENDPROPERTIES"#;

        assert_eq!(
            properties(&input.as_bytes()),
            Ok((EMPTY, Properties::new()))
        );
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

        assert_eq!(properties(&input.as_bytes()), Ok((EMPTY, expected)));
    }
}
