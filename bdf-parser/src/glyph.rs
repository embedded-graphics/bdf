use nom::{
    bytes::complete::{tag, take, take_until},
    character::complete::{multispace0, space0},
    combinator::{eof, map, map_parser, map_res, opt},
    multi::many0,
    sequence::{delimited, preceded, terminated},
    IResult,
};
use std::convert::TryFrom;

use crate::{helpers::*, BoundingBox, Coord};

/// Glyph encoding
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoding {
    /// Standard encoding
    Standard(u32),
    /// Non standard encoding
    NonStandard(u32),
    /// Unspecified encoding
    #[default]
    Unspecified,
}

/// Glyph.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Glyph {
    /// Name.
    pub name: String,

    /// Encoding.
    pub encoding: Encoding,

    /// Scalable width.
    pub scalable_width: Option<Coord>,

    /// Device width.
    pub device_width: Coord,

    /// Bounding box.
    pub bounding_box: BoundingBox,

    /// Bitmap data.
    pub bitmap: Vec<u8>,
}

impl Glyph {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, name) = statement("STARTCHAR", parse_string)(input)?;
        let (input, encoding) = statement("ENCODING", parse_encoding)(input)?;
        let (input, scalable_width) = opt(statement("SWIDTH", Coord::parse))(input)?;
        let (input, device_width) = statement("DWIDTH", Coord::parse)(input)?;
        let (input, bounding_box) = statement("BBX", BoundingBox::parse)(input)?;
        let (input, bitmap) = parse_bitmap(input)?;

        Ok((
            input,
            Self {
                name,
                encoding,
                scalable_width,
                device_width,
                bounding_box,
                bitmap,
            },
        ))
    }

    /// Returns a pixel from the bitmap.
    ///
    /// This method doesn't use the BDF coordinate system. The coordinates are relative to the
    /// top left corner of the bounding box and don't take the offset into account. Y coordinates
    /// increase downwards.
    ///
    /// Returns `None` if the coordinates are outside the bitmap.
    pub fn pixel(&self, x: usize, y: usize) -> Option<bool> {
        let width = usize::try_from(self.bounding_box.size.x).unwrap();

        if x >= width {
            return None;
        }

        let bytes_per_row = (width + 7) / 8;
        let byte_offset = x / 8;
        let bit_mask = 0x80 >> (x % 8);

        self.bitmap
            .get(byte_offset + bytes_per_row * y)
            .map(|v| v & bit_mask != 0)
    }

    /// Returns an iterator over the pixels in the glyph bitmap.
    ///
    /// Iteration starts at the top left corner of the bounding box and ends at the bottom right
    /// corner.
    pub fn pixels(&self) -> impl Iterator<Item = bool> + '_ {
        let width = usize::try_from(self.bounding_box.size.x).unwrap();
        let height = usize::try_from(self.bounding_box.size.y).unwrap();

        (0..height).flat_map(move |y| (0..width).map(move |x| self.pixel(x, y).unwrap()))
    }
}

fn parse_encoding(input: &[u8]) -> IResult<&[u8], Encoding> {
    let (input, standard_encoding) = parse_to_i32(input)?;
    let (input, non_standard_encoding) = opt(preceded(multispace0, parse_to_i32))(input)?;

    let encoding = if standard_encoding >= 0 {
        Encoding::Standard(u32::try_from(standard_encoding).unwrap())
    } else if let Some(non_standard) = non_standard_encoding {
        Encoding::NonStandard(u32::try_from(non_standard).unwrap())
    } else {
        Encoding::Unspecified
    };

    Ok((input, encoding))
}

fn parse_bitmap(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    map_parser(
        delimited(
            statement("BITMAP", eof),
            take_until("ENDCHAR"),
            statement("ENDCHAR", eof),
        ),
        preceded(multispace0, many0(terminated(parse_hex_byte, multispace0))),
    )(input)
}

fn parse_hex_byte(input: &[u8]) -> IResult<&[u8], u8> {
    map_res(map_res(take(2usize), std::str::from_utf8), |v| {
        u8::from_str_radix(v, 16)
    })(input)
}

/// Glyphs collection.
#[derive(Debug, Clone, PartialEq)]
pub struct Glyphs {
    glyphs: Vec<Glyph>,
}

impl Glyphs {
    pub(crate) fn new(mut glyphs: Vec<Glyph>) -> Self {
        glyphs.sort_by_key(|glyph| glyph.encoding);
        Self { glyphs }
    }

    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        map(
            preceded(
                terminated(opt(numchars), multispace0),
                many0(terminated(Glyph::parse, multispace0)),
            ),
            Self::new,
        )(input)
    }

    /// Gets a glyph by the encoding.
    pub fn get(&self, c: char) -> Option<&Glyph> {
        // TODO: this assumes that the font uses unicode
        let encoding = Encoding::Standard(c as u32);

        self.glyphs
            .binary_search_by_key(&encoding, |glyph| glyph.encoding)
            .map_or(None, |i| Some(&self.glyphs[i]))
    }

    /// Returns `true` if the collection contains the given character.
    pub fn contains(&self, c: char) -> bool {
        self.get(c).is_some()
    }

    /// Returns an iterator over all glyphs.
    pub fn iter(&self) -> impl Iterator<Item = &Glyph> {
        self.glyphs.iter()
    }
}

fn numchars(input: &[u8]) -> IResult<&[u8], u32> {
    preceded(
        space0,
        preceded(tag("CHARS"), preceded(space0, parse_to_u32)),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_parse_bitmap() {
        assert_parser_ok!(parse_bitmap(b"BITMAP\n7e\nENDCHAR"), vec![0x7e]);
        assert_parser_ok!(parse_bitmap(b"BITMAP\nff\nENDCHAR"), vec![0xff]);
        assert_parser_ok!(parse_bitmap(b"BITMAP\nCCCC\nENDCHAR"), vec![0xcc, 0xcc]);
        assert_parser_ok!(
            parse_bitmap(b"BITMAP\nffffffff\nENDCHAR"),
            vec![0xff, 0xff, 0xff, 0xff]
        );
        assert_parser_ok!(
            parse_bitmap(b"BITMAP\nffffffff\naaaaaaaa\nENDCHAR"),
            vec![0xff, 0xff, 0xff, 0xff, 0xaa, 0xaa, 0xaa, 0xaa]
        );
        assert_parser_ok!(
            parse_bitmap(b"BITMAP\nff\nff\nff\nff\naa\naa\naa\naa\nENDCHAR"),
            vec![0xff, 0xff, 0xff, 0xff, 0xaa, 0xaa, 0xaa, 0xaa]
        );
        assert_parser_ok!(
            parse_bitmap(
                b"BITMAP\n00\n00\n00\n00\n18\n24\n24\n42\n42\n7E\n42\n42\n42\n42\n00\n00\nENDCHAR"
            ),
            vec![
                0x00, 0x00, 0x00, 0x00, 0x18, 0x24, 0x24, 0x42, 0x42, 0x7e, 0x42, 0x42, 0x42, 0x42,
                0x00, 0x00
            ]
        );
    }

    /// Returns test data for a single glyph and the expected parsing result
    fn test_data() -> (&'static [u8], Glyph) {
        (
            indoc! {br#"
                STARTCHAR ZZZZ
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
                ENDCHAR
            "#},
            Glyph {
                name: "ZZZZ".to_string(),
                encoding: Encoding::Standard(65), // 'A'
                bitmap: vec![
                    0x00, 0x00, 0x00, 0x00, 0x18, 0x24, 0x24, 0x42, 0x42, 0x7e, 0x42, 0x42, 0x42,
                    0x42, 0x00, 0x00,
                ],
                bounding_box: BoundingBox {
                    size: Coord::new(8, 16),
                    offset: Coord::new(0, -2),
                },
                scalable_width: Some(Coord::new(500, 0)),
                device_width: Coord::new(8, 0),
            },
        )
    }

    #[test]
    fn parse_single_char() {
        let (chardata, expected_glyph) = test_data();

        assert_parser_ok!(Glyph::parse(chardata), expected_glyph);
    }

    #[test]
    fn get_glyph_by_char() {
        let (chardata, expected_glyph) = test_data();

        let (input, glyphs) = Glyphs::parse(chardata).unwrap();
        assert!(input.is_empty());
        assert_eq!(glyphs.get('A'), Some(&expected_glyph));
    }

    #[test]
    fn pixel_getter() {
        let (chardata, _) = test_data();
        let (input, glyph) = Glyph::parse(chardata).unwrap();
        assert!(input.is_empty());

        let bitmap = (0..16)
            .map(|y| {
                (0..8)
                    .map(|x| if glyph.pixel(x, y).unwrap() { '#' } else { ' ' })
                    .collect::<String>()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            bitmap,
            [
                "        ", //
                "        ", //
                "        ", //
                "        ", //
                "   ##   ", //
                "  #  #  ", //
                "  #  #  ", //
                " #    # ", //
                " #    # ", //
                " ###### ", //
                " #    # ", //
                " #    # ", //
                " #    # ", //
                " #    # ", //
                "        ", //
                "        ", //
            ]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn pixels_iterator() {
        let (chardata, _) = test_data();
        let (input, glyph) = Glyph::parse(chardata).unwrap();
        assert!(input.is_empty());

        let bitmap = glyph
            .pixels()
            .map(|v| if v { '#' } else { ' ' })
            .collect::<String>();

        assert_eq!(
            bitmap,
            concat!(
                "        ", //
                "        ", //
                "        ", //
                "        ", //
                "   ##   ", //
                "  #  #  ", //
                "  #  #  ", //
                " #    # ", //
                " #    # ", //
                " ###### ", //
                " #    # ", //
                " #    # ", //
                " #    # ", //
                " #    # ", //
                "        ", //
                "        ", //
            )
        );
    }

    #[test]
    fn pixel_getter_outside() {
        let (chardata, _) = test_data();
        let (input, glyph) = Glyph::parse(chardata).unwrap();
        assert!(input.is_empty());

        assert_eq!(glyph.pixel(8, 0), None);
        assert_eq!(glyph.pixel(0, 16), None);
        assert_eq!(glyph.pixel(8, 16), None);
    }

    #[test]
    fn parse_glyph_with_no_encoding() {
        let chardata = indoc! {br#"
            STARTCHAR 000
            ENCODING -1
            SWIDTH 432 0
            DWIDTH 6 0
            BBX 0 0 0 0
            BITMAP
            ENDCHAR
        "#};

        assert_parser_ok!(
            Glyph::parse(chardata),
            Glyph {
                bitmap: vec![],
                bounding_box: BoundingBox {
                    size: Coord::new(0, 0),
                    offset: Coord::new(0, 0),
                },
                encoding: Encoding::Unspecified,
                name: "000".to_string(),
                scalable_width: Some(Coord::new(432, 0)),
                device_width: Coord::new(6, 0),
            }
        );
    }

    #[test]
    fn parse_glyph_with_no_encoding_and_index() {
        let chardata = indoc! {br#"
            STARTCHAR 000
            ENCODING -1 123
            SWIDTH 432 0
            DWIDTH 6 0
            BBX 0 0 0 0
            BITMAP
            ENDCHAR
        "#};

        assert_parser_ok!(
            Glyph::parse(chardata),
            Glyph {
                bitmap: vec![],
                bounding_box: BoundingBox {
                    size: Coord::new(0, 0),
                    offset: Coord::new(0, 0),
                },
                encoding: Encoding::NonStandard(123),
                name: "000".to_string(),
                scalable_width: Some(Coord::new(432, 0)),
                device_width: Coord::new(6, 0),
            }
        );
    }

    #[test]
    fn parse_glyph_with_empty_bitmap() {
        let chardata = indoc! {br#"
            STARTCHAR 000
            ENCODING 0
            SWIDTH 432 0
            DWIDTH 6 0
            BBX 0 0 0 0
            BITMAP
            ENDCHAR
        "#};

        assert_parser_ok!(
            Glyph::parse(chardata),
            Glyph {
                bitmap: vec![],
                bounding_box: BoundingBox {
                    size: Coord::new(0, 0),
                    offset: Coord::new(0, 0),
                },
                encoding: Encoding::Standard(0),
                name: "000".to_string(),
                scalable_width: Some(Coord::new(432, 0)),
                device_width: Coord::new(6, 0),
            }
        );
    }
}
