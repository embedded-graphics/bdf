//! BDF parser.

#![deny(unsafe_code)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, space1},
    combinator::{eof, map, opt},
    sequence::separated_pair,
    IResult,
};

#[macro_use]
mod helpers;

mod glyph;
mod metadata;
mod properties;

pub use glyph::{Glyph, Glyphs};
use helpers::*;
pub use metadata::Metadata;
pub use properties::{Properties, Property, PropertyError};

/// BDF Font.
#[derive(Debug, Clone, PartialEq)]
pub struct BdfFont {
    /// Font metadata.
    pub metadata: Metadata,

    /// Glyphs.
    pub glyphs: Glyphs,

    /// Properties.
    pub properties: Properties,
}

impl BdfFont {
    /// Parses a BDF file.
    ///
    /// BDF files are expected to be ASCII encoded according to the BDF specification. Any non
    /// ASCII characters in strings will be replaced by the `U+FFFD` replacement character.
    pub fn parse(input: &[u8]) -> Result<Self, ParserError> {
        let (input, metadata) = Metadata::parse(input).map_err(|_| ParserError::Metadata)?;
        let input = skip_whitespace(input);
        let (input, properties) = Properties::parse(input).map_err(|_| ParserError::Properties)?;
        let input = skip_whitespace(input);
        let (input, glyphs) = Glyphs::parse(input).map_err(|_| ParserError::Glyphs)?;
        let input = skip_whitespace(input);
        let (input, _) = end_font(input).unwrap();
        let input = skip_whitespace(input);
        end_of_file(input).map_err(|_| ParserError::EndOfFile)?;

        Ok(Self {
            properties,
            metadata,
            glyphs,
        })
    }
}

fn skip_whitespace(input: &[u8]) -> &[u8] {
    multispace0::<_, nom::error::Error<_>>(input).unwrap().0
}

fn end_font(input: &[u8]) -> IResult<&[u8], Option<&[u8]>> {
    opt(tag("ENDFONT"))(input)
}

fn end_of_file(input: &[u8]) -> IResult<&[u8], &[u8]> {
    eof(input)
}

/// Bounding box.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BoundingBox {
    /// Size of the bounding box.
    pub size: Coord,

    /// Offset to the lower left corner of the bounding box.
    pub offset: Coord,
}

/// Coordinate.
///
/// BDF files use a cartesian coordinate system, where the positive half-axis points upwards.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Coord {
    /// X coordinate.
    pub x: i32,

    /// Y coordinate.
    pub y: i32,
}

impl Coord {
    /// Creates a new coord.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        map(
            separated_pair(parse_to_i32, space1, parse_to_i32),
            |(x, y)| Self::new(x, y),
        )(input)
    }
}

impl BoundingBox {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        map(
            separated_pair(Coord::parse, space1, Coord::parse),
            |(size, offset)| Self { size, offset },
        )(input)
    }
}

/// Parser error.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ParserError {
    /// Metadata.
    #[error("couldn't parse metadata")]
    Metadata,

    /// Properties.
    #[error("couldn't parse properties")]
    Properties,

    /// Glyphs.
    #[error("couldn't parse glyphs")]
    Glyphs,

    /// Unexpected input at the end of the file.
    #[error("unexpected input at the end of the file")]
    EndOfFile,
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    const FONT: &'static str = indoc! {r#"
        STARTFONT 2.1
        FONT "test font"
        SIZE 16 75 75
        FONTBOUNDINGBOX 16 24 0 0
        STARTPROPERTIES 3
        COPYRIGHT "Copyright123"
        FONT_ASCENT 1
        FONT_DESCENT 2
        ENDPROPERTIES
        STARTCHAR Char 0
        ENCODING 64
        DWIDTH 8 0
        BBX 8 8 0 0
        BITMAP
        1f
        01
        ENDCHAR
        STARTCHAR Char 1
        ENCODING 65
        DWIDTH 8 0
        BBX 8 8 0 0
        BITMAP
        2f
        02
        ENDCHAR
        ENDFONT
    "#};

    fn test_font(font: &BdfFont) {
        assert_eq!(
            font.metadata,
            Metadata {
                version: 2.1,
                name: String::from("\"test font\""),
                point_size: 16,
                resolution: Coord::new(75, 75),
                bounding_box: BoundingBox {
                    size: Coord::new(16, 24),
                    offset: Coord::new(0, 0),
                },
            }
        );

        assert_eq!(
            font.glyphs.iter().cloned().collect::<Vec<_>>(),
            vec![
                Glyph {
                    bitmap: vec![0x1f, 0x01],
                    bounding_box: BoundingBox {
                        size: Coord::new(8, 8),
                        offset: Coord::new(0, 0),
                    },
                    encoding: Some('@'), //64
                    name: "Char 0".to_string(),
                    device_width: Coord::new(8, 0),
                    scalable_width: None,
                },
                Glyph {
                    bitmap: vec![0x2f, 0x02],
                    bounding_box: BoundingBox {
                        size: Coord::new(8, 8),
                        offset: Coord::new(0, 0),
                    },
                    encoding: Some('A'), //65
                    name: "Char 1".to_string(),
                    device_width: Coord::new(8, 0),
                    scalable_width: None,
                },
            ],
        );

        assert_eq!(
            font.properties.try_get(Property::Copyright),
            Ok("Copyright123".to_string())
        );
        assert_eq!(font.properties.try_get(Property::FontAscent), Ok(1));
        assert_eq!(font.properties.try_get(Property::FontDescent), Ok(2));
    }

    #[test]
    fn parse_font() {
        test_font(&BdfFont::parse(FONT.as_bytes()).unwrap())
    }

    #[test]
    fn parse_font_without_endfont() {
        let lines: Vec<_> = FONT
            .lines()
            .filter(|line| !line.contains("ENDFONT"))
            .collect();
        let input = lines.join("\n");

        test_font(&BdfFont::parse(input.as_bytes()).unwrap());
    }

    #[test]
    fn parse_font_with_windows_line_endings() {
        let lines: Vec<_> = FONT.lines().collect();
        let input = lines.join("\r\n");

        test_font(&BdfFont::parse(input.as_bytes()).unwrap());
    }

    #[test]
    fn parse_font_with_garbage_after_endfont() {
        let lines: Vec<_> = FONT.lines().chain(std::iter::once("Invalid")).collect();
        let input = lines.join("\n");

        assert_eq!(
            BdfFont::parse(input.as_bytes()),
            Err(ParserError::EndOfFile)
        );
    }
}
