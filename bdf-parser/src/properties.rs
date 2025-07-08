use std::{collections::HashMap, convert::TryFrom};
use thiserror::Error;

use crate::parser::{Lines, ParserError};

/// BDF file property.
///
/// Source: <https://www.x.org/releases/X11R7.6/doc/xorg-docs/specs/XLFD/xlfd.html>
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
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Properties {
    properties: HashMap<String, PropertyValue>,
}

impl Properties {
    #[cfg(test)]
    pub(crate) fn new(properties: HashMap<String, PropertyValue>) -> Self {
        Self { properties }
    }

    pub(crate) fn parse(lines: &mut Lines<'_>) -> Result<Self, ParserError> {
        let start = lines.next().unwrap();
        assert_eq!(start.keyword, "STARTPROPERTIES");

        // TODO: check if number of properties is correct
        let _n_properties: usize = start
            .parameters
            .parse()
            .map_err(|_| ParserError::with_line("invalid \"STARTPROPERTIES\"", &start))?;

        let mut properties = HashMap::new();

        for line in lines {
            if line.keyword == "ENDPROPERTIES" {
                break;
            }

            let value = if let Ok(int) = line.parameters.parse::<i32>() {
                PropertyValue::Int(int)
            } else if let Some(text) = line
                .parameters
                .strip_prefix('"')
                .and_then(|p| p.strip_suffix('"'))
            {
                PropertyValue::Text(text.replace("\"\"", "\""))
            } else {
                return Err(ParserError::with_line("invalid property", &line));
            };

            properties.insert(line.keyword.to_string(), value);
        }

        Ok(Self { properties })
    }

    /// Tries to get a property.
    ///
    /// Returns `None` if the property doesn't exits and an error if the value has the wrong type.
    pub fn try_get<T: PropertyType>(
        &self,
        property: Property,
    ) -> Result<Option<T>, PropertyTypeError> {
        self.try_get_by_name(&property.to_string())
    }

    /// Tries to get a property by name.
    ///
    /// Returns `None` if the property doesn't exits and an error if the value has the wrong type.
    pub fn try_get_by_name<T: PropertyType>(
        &self,
        name: &str,
    ) -> Result<Option<T>, PropertyTypeError> {
        self.properties
            .get(name)
            .map(|value| value.try_into())
            .transpose()
    }

    /// Returns `true` if no properties exist.
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
}

/// Marker trait for property value types.
pub trait PropertyType
where
    Self: for<'a> TryFrom<&'a PropertyValue, Error = PropertyTypeError>,
{
}

impl PropertyType for String {}
impl PropertyType for i32 {}
impl PropertyType for u32 {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyValue {
    Text(String),
    Int(i32),
}

impl TryFrom<&PropertyValue> for String {
    type Error = PropertyTypeError;

    fn try_from(value: &PropertyValue) -> Result<Self, Self::Error> {
        match value {
            PropertyValue::Text(text) => Ok(text.clone()),
            _ => Err(PropertyTypeError),
        }
    }
}

impl TryFrom<&PropertyValue> for i32 {
    type Error = PropertyTypeError;

    fn try_from(value: &PropertyValue) -> Result<Self, Self::Error> {
        match value {
            PropertyValue::Int(int) => Ok(*int),
            _ => Err(PropertyTypeError),
        }
    }
}

impl TryFrom<&PropertyValue> for u32 {
    type Error = PropertyTypeError;

    fn try_from(value: &PropertyValue) -> Result<Self, Self::Error> {
        match value {
            PropertyValue::Int(int) if *int >= 0 => Ok(*int as u32),
            _ => Err(PropertyTypeError),
        }
    }
}

/// Invalid property type error.
#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
#[error("invalid property type")]
pub struct PropertyTypeError;

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn string_properties() {
        const INPUT: &str = indoc! {r#"
            STARTPROPERTIES 3
            KEY1 "VALUE"
            KEY2   "RANDOM WORDS AND STUFF"
            WITH_QUOTE "1""23"""
            ENDPROPERTIES
        "#};

        let mut lines = Lines::new(INPUT);
        let properties = Properties::parse(&mut lines).unwrap();

        for (key, expected) in [
            ("KEY1", "VALUE"),
            ("KEY2", "RANDOM WORDS AND STUFF"),
            ("WITH_QUOTE", "1\"23\""),
        ] {
            assert_eq!(
                properties.try_get_by_name::<String>(key).unwrap(),
                Some(expected.to_string()),
                "key=\"{key}\""
            );
        }
    }

    #[test]
    fn integer_properties() {
        const INPUT: &str = indoc! {r#"
            STARTPROPERTIES 2
            POS_INT 10
            NEG_INT -20
            ENDPROPERTIES
        "#};

        let mut lines = Lines::new(INPUT);
        let properties = Properties::parse(&mut lines).unwrap();

        assert_eq!(properties.try_get_by_name::<i32>("POS_INT"), Ok(Some(10)));
        assert_eq!(properties.try_get_by_name::<i32>("NEG_INT"), Ok(Some(-20)));

        assert_eq!(properties.try_get_by_name::<u32>("POS_INT"), Ok(Some(10)));
        assert_eq!(
            properties.try_get_by_name::<u32>("NEG_INT"),
            Err(PropertyTypeError)
        );

        assert_eq!(
            properties.try_get_by_name::<String>("POS_INT"),
            Err(PropertyTypeError)
        );
    }

    #[test]
    fn empty_properties() {
        const INPUT: &str = indoc! {r#"
            STARTPROPERTIES 0
            ENDPROPERTIES
        "#};

        let mut lines = Lines::new(INPUT);
        let properties = Properties::parse(&mut lines).unwrap();

        assert_eq!(properties.properties, HashMap::new());
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
