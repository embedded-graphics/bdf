use nom::{
    character::complete::{multispace0, space1},
    combinator::map_opt,
    sequence::{preceded, separated_pair},
    IResult, ParseTo,
};

use crate::{helpers::*, BoundingBox};

pub type FontSize = (i32, (u32, u32));

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub version: f32,
    pub name: String,
    pub size: FontSize,
    pub bounding_box: BoundingBox,
}
fn metadata_version(input: &[u8]) -> IResult<&[u8], f32> {
    map_opt(
        statement("STARTFONT", take_until_line_ending),
        |v: &[u8]| v.parse_to(),
    )(input)
}

fn metadata_name(input: &[u8]) -> IResult<&[u8], String> {
    map_opt(
        statement("FONT", take_until_line_ending),
        |name: &[u8]| name.parse_to(),
    )(input)
}

fn metadata_size(input: &[u8]) -> IResult<&[u8], FontSize> {
    statement("SIZE", separated_pair(parse_to_i32, space1, unsigned_xy))(input)
}

fn metadata_bounding_box(input: &[u8]) -> IResult<&[u8], BoundingBox> {
    statement(
        "FONTBOUNDINGBOX",
        separated_pair(unsigned_xy, space1, signed_xy),
    )(input)
}

pub fn header(input: &[u8]) -> IResult<&[u8], Metadata> {
    let (input, version) = preceded(optional_comments, metadata_version)(input)?;
    let (input, name) = preceded(optional_comments, metadata_name)(input)?;
    let (input, size) = preceded(optional_comments, metadata_size)(input)?;
    let (input, bounding_box) = preceded(optional_comments, metadata_bounding_box)(input)?;
    let (input, _) = optional_comments(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        Metadata {
            version,
            name,
            size,
            bounding_box,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: &[u8] = &[];

    #[test]
    fn it_parses_the_font_version() {
        assert_eq!(metadata_version(b"STARTFONT 2.1\n"), Ok((EMPTY, 2.1f32)));

        // Some fonts are a bit overzealous with their whitespace
        assert_eq!(metadata_version(b"STARTFONT   2.1\n"), Ok((EMPTY, 2.1f32)));
    }

    #[test]
    fn it_parses_header() {
        let input = r#"STARTFONT 2.1
FONT "test font"
SIZE 16 75 75
FONTBOUNDINGBOX 16 24 0 0"#;

        assert_eq!(
            header(input.as_bytes()),
            Ok((
                EMPTY,
                Metadata {
                    version: 2.1,
                    name: String::from("\"test font\""),
                    size: (16, (75, 75)),
                    bounding_box: ((16, 24), (0, 0))
                }
            ))
        );
    }
}
