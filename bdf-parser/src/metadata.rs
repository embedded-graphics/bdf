use nom::{
    character::complete::{multispace0, space1},
    combinator::map_opt,
    sequence::separated_pair,
    IResult, ParseTo,
};

use crate::{helpers::*, BoundingBox, Coord};

/// BDF file metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    /// BDF format version.
    pub version: f32,

    /// Font name.
    pub name: String,

    /// Point size.
    pub point_size: i32,

    /// X and Y resolution in DPI.
    pub resolution: Coord,

    /// Font bounding box.
    pub bounding_box: BoundingBox,
}

impl Metadata {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, version) = skip_comments(metadata_version)(input)?;
        let (input, name) = skip_comments(metadata_name)(input)?;
        let (input, (point_size, resolution)) = skip_comments(metadata_size)(input)?;
        let (input, bounding_box) = skip_comments(metadata_bounding_box)(input)?;
        let (input, _) = multispace0(input)?;

        Ok((
            input,
            Self {
                version,
                name,
                point_size,
                resolution,
                bounding_box,
            },
        ))
    }
}

fn metadata_version(input: &[u8]) -> IResult<&[u8], f32> {
    map_opt(statement("STARTFONT", parse_string), |v: String| {
        v.as_str().parse_to()
    })(input)
}

fn metadata_name(input: &[u8]) -> IResult<&[u8], String> {
    statement("FONT", parse_string)(input)
}

fn metadata_size(input: &[u8]) -> IResult<&[u8], (i32, Coord)> {
    statement("SIZE", separated_pair(parse_to_i32, space1, Coord::parse))(input)
}

fn metadata_bounding_box(input: &[u8]) -> IResult<&[u8], BoundingBox> {
    statement("FONTBOUNDINGBOX", BoundingBox::parse)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bdf_version() {
        assert_parser_ok!(metadata_version(b"STARTFONT 2.1\n"), 2.1f32);

        // Some fonts are a bit overzealous with their whitespace
        assert_parser_ok!(metadata_version(b"STARTFONT   2.1\n"), 2.1f32);
    }

    #[test]
    fn parse_font_name() {
        assert_parser_ok!(metadata_name(b"FONT abc"), "abc".to_string());
    }

    #[test]
    fn parse_metadata() {
        let input = br#"STARTFONT 2.1
FONT "test font"
SIZE 16 75 100
FONTBOUNDINGBOX 16 24 1 2"#;

        assert_parser_ok!(
            Metadata::parse(input),
            Metadata {
                version: 2.1,
                name: String::from("\"test font\""),
                point_size: 16,
                resolution: Coord::new(75, 100),
                bounding_box: BoundingBox {
                    size: Coord::new(16, 24),
                    offset: Coord::new(1, 2),
                }
            }
        );
    }
}
