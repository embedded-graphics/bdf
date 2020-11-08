use nom::{
    bytes::complete::{tag, take_until},
    character::{complete::multispace0, is_hex_digit},
    combinator::{map_opt, opt},
    sequence::delimited,
    IResult, ParseTo,
};

use crate::{helpers::*, BoundingBox};

type Vec2 = (u32, u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub name: String,
    pub charcode: i32,
    pub bounding_box: BoundingBox,
    pub bitmap: Vec<u32>,
}

fn glyph_startchar(input: &[u8]) -> IResult<&[u8], String> {
    statement(
        "STARTCHAR",
        map_opt(take_until_line_ending, |name| name.parse_to()),
    )(input)
}

fn glyph_charcode(input: &[u8]) -> IResult<&[u8], i32> {
    statement("ENCODING", parse_to_i32)(input)
}

fn glyph_dwidth(input: &[u8]) -> IResult<&[u8], Vec2> {
    statement("DWIDTH", unsigned_xy)(input)
}

fn glyph_swidth(input: &[u8]) -> IResult<&[u8], Vec2> {
    statement("SWIDTH", unsigned_xy)(input)
}

fn glyph_bounding_box(input: &[u8]) -> IResult<&[u8], BoundingBox> {
    statement("BBX", BoundingBox::parse)(input)
}

fn glyph_bitmap(input: &[u8]) -> IResult<&[u8], Vec<u32>> {
    let (input, _) = multispace0(input)?;
    let (input, glyph_data) =
        delimited(tag("BITMAP"), take_until("ENDCHAR"), tag("ENDCHAR"))(input)?;

    Ok((
        input,
        glyph_data
            .to_vec()
            .iter()
            .filter(|d| is_hex_digit(**d))
            .collect::<Vec<&u8>>()
            .chunks(8)
            .map(|c| {
                c.iter()
                    .rev()
                    .enumerate()
                    .map(|(k, &&v)| {
                        let digit = v as char;
                        digit.to_digit(16).unwrap_or(0) << (k * 4)
                    })
                    .sum()
            })
            .collect(),
    ))
}

pub fn glyph(input: &[u8]) -> IResult<&[u8], Glyph> {
    let (input, name) = glyph_startchar(input)?;
    let (input, charcode) = glyph_charcode(input)?;
    let (input, _) = opt(glyph_swidth)(input)?;
    let (input, _) = opt(glyph_dwidth)(input)?;
    let (input, bounding_box) = glyph_bounding_box(input)?;
    let (input, bitmap) = glyph_bitmap(input)?;

    Ok((
        input,
        Glyph {
            bitmap,
            bounding_box,
            charcode,
            name,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: &[u8] = &[];

    #[test]
    fn it_parses_bitmap_data() {
        assert_eq!(
            glyph_bitmap(b"BITMAP\n7e\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![0x7e]))
        );
        assert_eq!(
            glyph_bitmap(b"BITMAP\nff\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![255]))
        );
        assert_eq!(
            glyph_bitmap(b"BITMAP\nCCCC\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![0xcccc]))
        );
        assert_eq!(
            glyph_bitmap(b"BITMAP\nffffffff\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![0xffffffff]))
        );
        assert_eq!(
            glyph_bitmap(b"BITMAP\nffffffff\naaaaaaaa\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![0xffffffff, 0xaaaaaaaa]))
        );
        assert_eq!(
            glyph_bitmap(b"BITMAP\nff\nff\nff\nff\naa\naa\naa\naa\nENDCHAR".as_ref()),
            Ok((EMPTY, vec![0xffffffff, 0xaaaaaaaa]))
        );
        assert_eq!(
            glyph_bitmap(
                b"BITMAP\n00\n00\n00\n00\n18\n24\n24\n42\n42\n7E\n42\n42\n42\n42\n00\n00\nENDCHAR"
                    .as_ref()
            ),
            Ok((EMPTY, vec![0x00000000, 0x18242442, 0x427e4242, 0x42420000]))
        );
    }

    #[test]
    fn it_parses_a_single_char() {
        let chardata = r#"STARTCHAR ZZZZ
ENCODING 65
SWIDTH 500 0
DWIDTH 8 0
BBX 8 16 0 -2
BITMAP
00
00
00
00
18
24
24
42
42
7E
42
42
42
42
00
00
ENDCHAR"#;

        let out = glyph(chardata.as_bytes());

        assert_eq!(
            out,
            Ok((
                EMPTY,
                Glyph {
                    name: "ZZZZ".to_string(),
                    charcode: 65,
                    bitmap: vec![0x00000000, 0x18242442, 0x427e4242, 0x42420000],
                    bounding_box: BoundingBox {
                        size: (8, 16),
                        offset: (0, -2),
                    },
                }
            ))
        );
    }

    #[test]
    fn it_parses_negative_encodings() {
        let chardata = r#"STARTCHAR 000
ENCODING -1
SWIDTH 432 0
DWIDTH 6 0
BBX 0 0 0 0
BITMAP
ENDCHAR"#;

        let out = glyph(chardata.as_bytes());

        assert_eq!(
            out,
            Ok((
                EMPTY,
                Glyph {
                    bitmap: vec![],
                    bounding_box: BoundingBox {
                        size: (0, 0),
                        offset: (0, 0),
                    },
                    charcode: -1i32,
                    name: "000".to_string(),
                }
            ))
        );
    }

    #[test]
    fn it_parses_chars_with_no_bitmap() {
        let chardata = r#"STARTCHAR 000
ENCODING 0
SWIDTH 432 0
DWIDTH 6 0
BBX 0 0 0 0
BITMAP
ENDCHAR"#;

        let out = glyph(chardata.as_bytes());

        assert_eq!(
            out,
            Ok((
                EMPTY,
                Glyph {
                    bitmap: vec![],
                    bounding_box: BoundingBox {
                        size: (0, 0),
                        offset: (0, 0),
                    },
                    charcode: 0,
                    name: "000".to_string(),
                }
            ))
        );
    }
}
