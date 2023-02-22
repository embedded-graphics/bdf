use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{multispace0, space1},
    combinator::{eof, map, map_opt, map_parser, opt},
    multi::{many0, many1},
    sequence::delimited,
    IResult, ParseTo,
};
use std::{collections::HashMap, convert::TryFrom};
use thiserror::Error;

use crate::helpers::*;

/// BDF file property.
///
/// Source: https://www.x.org/releases/X11R7.6/doc/xorg-docs/specs/XLFD/xlfd.html
#[derive(Debug, PartialEq, Copy, Clone, Eq, PartialOrd, Ord, strum::Display)]
#[strum(serialize_all = "shouty_snake_case")]
pub enum Property {
    /// ADD_STYLE_NAME
    AddStyleName,
    /// AVERAGE_WIDTH
    AverageWidth,
    /// AVG_CAPITAL_WIDTH
    AvgCapitalWidth,
    /// AVG_LOWERCASE_WIDTH
    AvgLowercaseWidth,
    /// AXIS_LIMITS
    AxisLimits,
    /// AXIS_NAMES
    AxisNames,
    /// AXIS_TYPES
    AxisTypes,
    /// CAP_HEIGHT
    CapHeight,
    /// CHARSET_ENCODING
    CharsetEncoding,
    /// CHARSET_REGISTRY
    CharsetRegistry,
    /// COPYRIGHT
    Copyright,
    /// DEFAULT_CHAR
    DefaultChar,
    /// DESTINATION
    Destination,
    /// END_SPACE
    EndSpace,
    /// FACE_NAME
    FaceName,
    /// FAMILY_NAME
    FamilyName,
    /// FIGURE_WIDTH
    FigureWidth,
    /// FONT
    Font,
    /// FONT_ASCENT
    FontAscent,
    /// FONT_DESCENT
    FontDescent,
    /// FONT_TYPE
    FontType,
    /// FONT_VERSION
    FontVersion,
    /// FOUNDRY
    Foundry,
    /// FULL_NAME
    FullName,
    /// ITALIC_ANGLE
    ItalicAngle,
    /// MAX_SPACE
    MaxSpace,
    /// MIN_SPACE
    MinSpace,
    /// NORM_SPACE
    NormSpace,
    /// NOTICE
    Notice,
    /// PIXEL_SIZE
    PixelSize,
    /// POINT_SIZE
    PointSize,
    /// QUAD_WIDTH
    QuadWidth,
    /// RASTERIZER_NAME
    RasterizerName,
    /// RASTERIZER_VERSION
    RasterizerVersion,
    /// RAW_ASCENT
    RawAscent,
    /// RAW_DESCENT
    RawDescent,
    /// RELATIVE_SETWIDTH
    RelativeSetwidth,
    /// RELATIVE_WEIGHT
    RelativeWeight,
    /// RESOLUTION
    Resolution,
    /// RESOLUTION_X
    ResolutionX,
    /// RESOLUTION_Y
    ResolutionY,
    /// SETWIDTH_NAME
    SetwidthName,
    /// SLANT
    Slant,
    /// SMALL_CAP_SIZE
    SmallCapSize,
    /// SPACING
    Spacing,
    /// STRIKEOUT_ASCENT
    StrikeoutAscent,
    /// STRIKEOUT_DESCENT
    StrikeoutDescent,
    /// SUBSCRIPT_SIZE
    SubscriptSize,
    /// SUBSCRIPT_X
    SubscriptX,
    /// SUBSCRIPT_Y
    SubscriptY,
    /// SUPERSCRIPT_SIZE
    SuperscriptSize,
    /// SUPERSCRIPT_X
    SuperscriptX,
    /// SUPERSCRIPT_Y
    SuperscriptY,
    /// UNDERLINE_POSITION
    UnderlinePosition,
    /// UNDERLINE_THICKNESS
    UnderlineThickness,
    /// WEIGHT
    Weight,
    /// WEIGHT_NAME
    WeightName,
    /// X_HEIGHT
    XHeight,
}

/// BDF file properties.
#[derive(Debug, Clone, PartialEq)]
pub struct Properties {
    properties: HashMap<String, PropertyValue>,
}

impl Properties {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        map(
            opt(map_parser(
                delimited(
                    statement("STARTPROPERTIES", parse_to_u32),
                    take_until("ENDPROPERTIES"),
                    statement("ENDPROPERTIES", eof),
                ),
                many0(property),
            )),
            |properties| {
                // Convert vector of properties into a HashMap
                let properties = properties
                    .map(|p| p.iter().cloned().collect())
                    .unwrap_or_else(HashMap::new);

                Self { properties }
            },
        )(input)
    }

    /// Tries to get a property.
    ///
    /// Returns an error if the property doesn't exist or the value has the wrong type.
    pub fn try_get<T>(&self, property: Property) -> Result<T, PropertyError>
    where
        T: for<'a> TryFrom<&'a PropertyValue, Error = PropertyError>,
    {
        self.try_get_by_name(&property.to_string())
    }

    /// Tries to get a property by name.
    ///
    /// Returns an error if the property doesn't exist or the value has the wrong type.
    pub fn try_get_by_name<T>(&self, name: &str) -> Result<T, PropertyError>
    where
        T: for<'a> TryFrom<&'a PropertyValue, Error = PropertyError>,
    {
        self.properties
            .get(name)
            .ok_or_else(|| PropertyError::Undefined(name.to_string()))
            .and_then(TryFrom::try_from)
    }

    /// Returns `true` if no properties exist.
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
}

fn property(input: &[u8]) -> IResult<&[u8], (String, PropertyValue)> {
    let (input, _) = multispace0(input)?;
    let (input, key) = map_opt(take_until(" "), |s: &[u8]| s.parse_to())(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = PropertyValue::parse(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, (key, value)))
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Text(String),
    Int(i32),
}

impl PropertyValue {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        alt((Self::parse_string, Self::parse_int))(input)
    }

    fn parse_string(input: &[u8]) -> IResult<&[u8], PropertyValue> {
        map(
            many1(delimited(
                tag("\""),
                map(take_until("\""), ascii_to_string_lossy),
                tag("\""),
            )),
            |parts| PropertyValue::Text(parts.join("\"")),
        )(input)
    }

    fn parse_int(input: &[u8]) -> IResult<&[u8], PropertyValue> {
        map(parse_to_i32, PropertyValue::Int)(input)
    }
}

impl TryFrom<&PropertyValue> for String {
    type Error = PropertyError;

    fn try_from(value: &PropertyValue) -> Result<Self, Self::Error> {
        match value {
            PropertyValue::Text(text) => Ok(text.clone()),
            _ => Err(PropertyError::WrongType),
        }
    }
}

impl TryFrom<&PropertyValue> for i32 {
    type Error = PropertyError;

    fn try_from(value: &PropertyValue) -> Result<Self, Self::Error> {
        match value {
            PropertyValue::Int(int) => Ok(*int),
            _ => Err(PropertyError::WrongType),
        }
    }
}

/// Error returned by property getters.
#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertyError {
    /// Undefined property.
    #[error("property \"{0}\" is undefined")]
    Undefined(String),
    /// Wrong property type.
    #[error("wrong property type")]
    WrongType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn parse_property_with_whitespace() {
        assert_parser_ok!(
            property(b"KEY   \"VALUE\""),
            ("KEY".to_string(), PropertyValue::Text("VALUE".to_string()))
        );

        assert_parser_ok!(
            property(b"KEY   \"RANDOM WORDS AND STUFF\""),
            (
                "KEY".to_string(),
                PropertyValue::Text("RANDOM WORDS AND STUFF".to_string())
            )
        );
    }

    #[test]
    fn parse_string_property() {
        assert_parser_ok!(
            property(b"KEY \"VALUE\""),
            ("KEY".to_string(), PropertyValue::Text("VALUE".to_string()))
        );
    }

    #[test]
    fn parse_string_property_with_quote_in_value() {
        assert_parser_ok!(
            property(br#"WITH_QUOTE "1""23""""#),
            (
                "WITH_QUOTE".to_string(),
                PropertyValue::Text("1\"23\"".to_string())
            )
        );
    }

    #[test]
    fn parse_string_property_with_invalid_ascii() {
        assert_parser_ok!(
            property(b"KEY \"VALUE\xAB\""),
            (
                "KEY".to_string(),
                PropertyValue::Text("VALUE\u{FFFD}".to_string())
            )
        );
    }

    #[test]
    fn parse_integer_property() {
        assert_parser_ok!(
            property(b"POSITIVE_NUMBER 10"),
            ("POSITIVE_NUMBER".to_string(), PropertyValue::Int(10i32))
        );

        assert_parser_ok!(
            property(b"NEGATIVE_NUMBER -10"),
            ("NEGATIVE_NUMBER".to_string(), PropertyValue::Int(-10i32))
        );
    }

    #[test]
    fn parse_empty_property_list() {
        let input = indoc! {br#"
            STARTPROPERTIES 0
            ENDPROPERTIES
        "#};

        let (input, properties) = Properties::parse(input).unwrap();
        assert_eq!(input, b"");
        assert!(properties.is_empty());
    }

    #[test]
    fn parse_properties() {
        let input = indoc! {br#"
            STARTPROPERTIES 2
            TEXT "FONT"
            INTEGER 10
            ENDPROPERTIES
        "#};

        let (input, properties) = Properties::parse(input).unwrap();
        assert_eq!(input, b"");

        assert_eq!(properties.properties.len(), 2);
        assert_eq!(properties.try_get_by_name("TEXT"), Ok("FONT".to_string()));
        assert_eq!(properties.try_get_by_name("INTEGER"), Ok(10));
    }

    #[test]
    fn try_get() {
        let input = indoc! {br#"
            STARTPROPERTIES 2
            FAMILY_NAME "FAMILY"
            RESOLUTION_X 100
            RESOLUTION_Y 75
            ENDPROPERTIES
        "#};

        let (input, properties) = Properties::parse(input).unwrap();
        assert_eq!(input, b"");

        assert_eq!(properties.properties.len(), 3);
        assert_eq!(
            properties.try_get(Property::FamilyName),
            Ok("FAMILY".to_string())
        );
        assert_eq!(properties.try_get(Property::ResolutionX), Ok(100));
        assert_eq!(properties.try_get(Property::ResolutionY), Ok(75));
    }

    #[test]
    fn property_to_string() {
        assert_eq!(&Property::Font.to_string(), "FONT");
        assert_eq!(&Property::SuperscriptX.to_string(), "SUPERSCRIPT_X");
        assert_eq!(
            &Property::AvgLowercaseWidth.to_string(),
            "AVG_LOWERCASE_WIDTH"
        );
    }
}
