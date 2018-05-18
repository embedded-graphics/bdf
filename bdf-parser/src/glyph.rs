use nom::types::CompleteByteSlice;
use nom::*;

use super::BoundingBox;
use super::helpers::*;

type Vec2 = (u32, u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub name: String,
    pub charcode: i32,
    pub bounding_box: BoundingBox,
    pub bitmap: Vec<u32>,
}

named!(
    glyph_name<CompleteByteSlice, String>,
    flat_map!(recognize!(take_until_line_ending), parse_to!(String))
);

named!(
    glyph_charcode<CompleteByteSlice, i32>,
    ws!(preceded!(tag!("ENCODING"), parse_to_i32))
);

named!(
    glyph_dwidth<CompleteByteSlice, Vec2>,
    ws!(preceded!(
        tag!("DWIDTH"),
        tuple!(parse_to_u32, parse_to_u32)
    ))
);

named!(
    glyph_swidth<CompleteByteSlice, Vec2>,
    ws!(preceded!(
        tag!("SWIDTH"),
        tuple!(parse_to_u32, parse_to_u32)
    ))
);

named!(
    glyph_bounding_box<CompleteByteSlice, BoundingBox>,
    ws!(preceded!(
        tag!("BBX"),
        tuple!(parse_to_u32, parse_to_u32, parse_to_i32, parse_to_i32)
    ))
);

named!(
    glyph_bitmap<CompleteByteSlice, Vec<u32>>,
    map!(
        ws!(delimited!(
            tag!("BITMAP"),
            take_until!("ENDCHAR"),
            tag!("ENDCHAR")
        )),
        |res| {
            res.to_vec()
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
                .collect()
        }
    )
);

named!(
    pub glyph<CompleteByteSlice, Glyph>,
    ws!(preceded!(
        tag!("STARTCHAR"),
        do_parse!(
            name: glyph_name >> charcode: glyph_charcode >> opt!(glyph_swidth) >> opt!(glyph_dwidth)
                >> bounding_box: glyph_bounding_box >> bitmap: glyph_bitmap >> ({
                Glyph {
                    bitmap,
                    bounding_box,
                    charcode,
                    name,
                }
            })
        )
    ))
);

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: CompleteByteSlice = CompleteByteSlice(b"");

    #[test]
    fn it_parses_bitmap_data() {
        assert_eq!(
            glyph_bitmap(CompleteByteSlice(b"BITMAP\n7e\nENDCHAR")),
            Ok((EMPTY, vec![0x7e]))
        );
        assert_eq!(
            glyph_bitmap(CompleteByteSlice(b"BITMAP\nff\nENDCHAR")),
            Ok((EMPTY, vec![255]))
        );
        assert_eq!(
            glyph_bitmap(CompleteByteSlice(b"BITMAP\nCCCC\nENDCHAR")),
            Ok((EMPTY, vec![0xcccc]))
        );
        assert_eq!(
            glyph_bitmap(CompleteByteSlice(b"BITMAP\nffffffff\nENDCHAR")),
            Ok((EMPTY, vec![0xffffffff]))
        );
        assert_eq!(
            glyph_bitmap(CompleteByteSlice(b"BITMAP\nffffffff\naaaaaaaa\nENDCHAR")),
            Ok((EMPTY, vec![0xffffffff, 0xaaaaaaaa]))
        );
        assert_eq!(
            glyph_bitmap(CompleteByteSlice(
                b"BITMAP\nff\nff\nff\nff\naa\naa\naa\naa\nENDCHAR"
            )),
            Ok((EMPTY, vec![0xffffffff, 0xaaaaaaaa]))
        );
        assert_eq!(
            glyph_bitmap(CompleteByteSlice(
                b"BITMAP\n00\n00\n00\n00\n18\n24\n24\n42\n42\n7E\n42\n42\n42\n42\n00\n00\nENDCHAR"
            )),
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

        let out = glyph(CompleteByteSlice(chardata.as_bytes()));

        assert_eq!(
            out,
            Ok((
                EMPTY,
                Glyph {
                    name: "ZZZZ".to_string(),
                    charcode: 65,
                    bitmap: vec![0x00000000, 0x18242442, 0x427e4242, 0x42420000],
                    bounding_box: (8, 16, 0, -2),
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

        let out = glyph(CompleteByteSlice(chardata.as_bytes()));

        assert_eq!(
            out,
            Ok((
                EMPTY,
                Glyph {
                    bitmap: vec![],
                    bounding_box: (0, 0, 0, 0),
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

        let out = glyph(CompleteByteSlice(chardata.as_bytes()));

        assert_eq!(
            out,
            Ok((
                EMPTY,
                Glyph {
                    bitmap: vec![],
                    bounding_box: (0, 0, 0, 0),
                    charcode: 0,
                    name: "000".to_string(),
                }
            ))
        );
    }
}
