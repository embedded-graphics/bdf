use nom::types::CompleteByteSlice;
use nom::*;
use std::collections::HashMap;

use super::helpers::*;

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Text(String),
    Int(i32),
}

pub type Properties = HashMap<String, PropertyValue>;

named!(property_value_string<CompleteByteSlice, PropertyValue>,
    map!(
        flat_map!(
            ws!(delimited!(tag!("\""), take_until!("\""), tag!("\""))),
            parse_to!(String)
        ),
        |str| PropertyValue::Text(str)
    )
);

named!(property_value_int<CompleteByteSlice, PropertyValue>,
    ws!(map!(
        parse_to_i32,
        |num| PropertyValue::Int(num)
    ))
);

named!(property_value<CompleteByteSlice, PropertyValue>, ws!(alt!(
    property_value_string |
    property_value_int
)));

named!(property<CompleteByteSlice, (String, PropertyValue)>,
    ws!(do_parse!(
        key: flat_map!(take_until!(" "), parse_to!(String)) >> value: property_value >> ({
            (key, value)
        })
    ))
);

named!(num_properties<CompleteByteSlice, u32>,
    flat_map!(
        ws!(preceded!(tag!("STARTPROPERTIES"), digit)),
        parse_to!(u32)
    )
);

named!(
    pub properties<CompleteByteSlice, Properties>,
    map!(
        flat_map!(
            delimited!(
                num_properties,
                take_until!("ENDPROPERTIES"),
                tag!("ENDPROPERTIES")
            ),
            many0!(property)
        ),
        |res| {
            res.iter().cloned().collect::<Properties>()
        }
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: CompleteByteSlice = CompleteByteSlice(b"");

    #[test]
    fn it_parses_whitespacey_properties() {
        assert_eq!(
            property(CompleteByteSlice(b"KEY   \"VALUE\"")),
            Ok((
                EMPTY,
                ("KEY".to_string(), PropertyValue::Text("VALUE".to_string()))
            ))
        );

        assert_eq!(
            property(CompleteByteSlice(b"KEY   \"RANDOM WORDS AND STUFF\"")),
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
            property(CompleteByteSlice(b"KEY \"VALUE\"")),
            Ok((
                EMPTY,
                ("KEY".to_string(), PropertyValue::Text("VALUE".to_string()))
            ))
        );
    }

    #[test]
    fn it_parses_integer_properties() {
        assert_eq!(
            property(CompleteByteSlice(b"POSITIVE_NUMBER 10")),
            Ok((
                EMPTY,
                ("POSITIVE_NUMBER".to_string(), PropertyValue::Int(10i32))
            ))
        );

        assert_eq!(
            property(CompleteByteSlice(b"NEGATIVE_NUMBER -10")),
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
            properties(CompleteByteSlice(&input.as_bytes())),
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

        assert_eq!(
            properties(CompleteByteSlice(&input.as_bytes())),
            Ok((EMPTY, expected))
        );
    }
}
