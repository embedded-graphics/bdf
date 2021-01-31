use nom::{
    character::complete::{multispace0, space1},
    combinator::{map, map_opt},
    sequence::separated_pair,
    IResult, ParseTo,
};

use crate::{helpers::*, BoundingBox, Coord};

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub version: f32,
    pub name: String,
    pub point_size: i32,
    pub resolution: Coord,
    pub bounding_box: BoundingBox,
}

impl Metadata {
    pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
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

fn metadata_version(input: &str) -> IResult<&str, f32> {
    map_opt(statement("STARTFONT", take_until_line_ending), |v: &str| {
        v.parse_to()
    })(input)
}

fn metadata_name(input: &str) -> IResult<&str, String> {
    map(statement("FONT", take_until_line_ending), String::from)(input)
}

fn metadata_size(input: &str) -> IResult<&str, (i32, Coord)> {
    statement("SIZE", separated_pair(parse_to_i32, space1, Coord::parse))(input)
}

fn metadata_bounding_box(input: &str) -> IResult<&str, BoundingBox> {
    statement("FONTBOUNDINGBOX", BoundingBox::parse)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_the_font_version() {
        assert_eq!(metadata_version("STARTFONT 2.1\n"), Ok(("", 2.1f32)));

        // Some fonts are a bit overzealous with their whitespace
        assert_eq!(metadata_version("STARTFONT   2.1\n"), Ok(("", 2.1f32)));
    }

    #[test]
    fn it_parses_header() {
        let input = r#"STARTFONT 2.1
FONT "test font"
SIZE 16 75 100
FONTBOUNDINGBOX 16 24 1 2"#;

        assert_eq!(
            Metadata::parse(input),
            Ok((
                "",
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
            ))
        );
    }
}
