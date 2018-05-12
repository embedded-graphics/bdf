#[macro_use]
extern crate nom;

use nom::*;

pub type FontSize = (i32, u32, u32);
pub type BoundingBox = (u32, u32, i32, i32);
type Vec2 = (u32, u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    name: String,
    charcode: i32,
    bounding_box: BoundingBox,
    bitmap: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
struct Properties;

#[derive(Debug, Clone, PartialEq)]
struct Metadata {
    version: f32,
    name: String,
    size: FontSize,
    bounding_box: BoundingBox,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BDFFont {
    metadata: Metadata,
    glyphs: Vec<Glyph>,
}

pub struct BDFParser<'a> {
    source: &'a str,
}

impl<'a> BDFParser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn parse(&self) -> nom::IResult<&[u8], BDFFont> {
        bdf(self.source.as_bytes())
    }
}

//
// HELPERS
//
named!(
    parse_to_i32<i32>,
    flat_map!(
        recognize!(preceded!(opt!(one_of!("+-")), digit)),
        parse_to!(i32)
    )
);

named!(
    parse_to_u32<u32>,
    flat_map!(recognize!(digit), parse_to!(u32))
);

//
// PROPERTIES
//
named!(
    properties<Properties>,
    map!(
        ws!(delimited!(
            tag!("STARTPROPERTIES"),
            take_until!("ENDPROPERTIES"),
            tag!("ENDPROPERTIES")
        )),
        |_| Properties
    )
);

//
// GLYPH
//
named!(
    glyph_name<String>,
    flat_map!(take_until!("\n"), parse_to!(String))
);

named!(
    glyph_charcode<i32>,
    ws!(preceded!(tag!("ENCODING"), parse_to_i32))
);

named!(
    glyph_dwidth<Vec2>,
    ws!(preceded!(
        tag!("DWIDTH"),
        tuple!(parse_to_u32, parse_to_u32)
    ))
);

named!(
    glyph_swidth<Vec2>,
    ws!(preceded!(
        tag!("SWIDTH"),
        tuple!(parse_to_u32, parse_to_u32)
    ))
);

named!(
    glyph_bounding_box<BoundingBox>,
    ws!(preceded!(
        tag!("BBX"),
        tuple!(parse_to_u32, parse_to_u32, parse_to_i32, parse_to_i32)
    ))
);

named!(
    glyph_bitmap<Vec<u32>>,
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
    glyph<Glyph>,
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

//
// METADATA
///
named!(
    metadata_version<f32>,
    flat_map!(
        preceded!(tag!("STARTFONT "), take_until!("\n")),
        parse_to!(f32)
    )
);

named!(
    metadata_name<String>,
    flat_map!(
        preceded!(tag!("FONT "), take_until!("\n")),
        parse_to!(String)
    )
);

named!(
    metadata_size<FontSize>,
    ws!(preceded!(
        tag!("SIZE"),
        tuple!(parse_to_i32, parse_to_u32, parse_to_u32)
    ))
);

named!(
    metadata_bounding_box<BoundingBox>,
    ws!(preceded!(
        tag!("FONTBOUNDINGBOX"),
        tuple!(parse_to_u32, parse_to_u32, parse_to_i32, parse_to_i32)
    ))
);

named!(
    header<Metadata>,
    ws!(do_parse!(
        version: metadata_version >> name: metadata_name >> size: metadata_size
            >> bounding_box: metadata_bounding_box >> ({
            Metadata {
                version,
                name,
                size,
                bounding_box,
            }
        })
    ))
);

named!(
    inner_bdf<BDFFont>,
    ws!(do_parse!(
        header >> properties >> glyphs: many0!(glyph) >> ({
            BDFFont {
                metadata: Metadata {
                    version: 2.1,
                    name: String::from("\"open_iconic_all_1x\""),
                    size: (16, 75, 75),
                    bounding_box: (16, 16, 0, 0),
                },
                glyphs,
            }
        })
    ))
);

named!(
    bdf<BDFFont>,
    alt_complete!(ws!(terminated!(inner_bdf, tag!("ENDFONT"))) | inner_bdf)
);

pub fn parse_char(input: &str) -> nom::IResult<&[u8], Glyph> {
    glyph(input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult;

    const EMPTY: &[u8] = &[];

    #[test]
    fn it_parses_a_font_file() {
        let chardata = r#"STARTFONT 2.1
FONT "open_iconic_all_1x"
SIZE 16 75 75
FONTBOUNDINGBOX 16 16 0 0
STARTPROPERTIES 3
COPYRIGHT "https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE"
FONT_ASCENT 0
FONT_DESCENT 0
ENDPROPERTIES
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
1f
01
ENDCHAR
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
2f
02
ENDCHAR
ENDFONT
"#;

        let out = bdf(&chardata.as_bytes());

        assert_eq!(
            out,
            IResult::Done(
                EMPTY,
                BDFFont {
                    metadata: Metadata {
                        version: 2.1,
                        name: String::from("\"open_iconic_all_1x\""),
                        size: (16, 75, 75),
                        bounding_box: (16, 16, 0, 0),
                    },
                    glyphs: vec![
                        Glyph {
                            bitmap: vec![0x1f01],
                            bounding_box: (8, 8, 0, 0),
                            charcode: 64,
                            name: "000".to_string(),
                        },
                        Glyph {
                            bitmap: vec![0x2f02],
                            bounding_box: (8, 8, 0, 0),
                            charcode: 64,
                            name: "000".to_string(),
                        },
                    ],
                }
            )
        );
    }

    #[test]
    fn it_parses_optional_endfont_tag() {
        let chardata = r#"STARTFONT 2.1
FONT "open_iconic_all_1x"
SIZE 16 75 75
FONTBOUNDINGBOX 16 16 0 0
STARTPROPERTIES 3
COPYRIGHT "https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE"
FONT_ASCENT 0
FONT_DESCENT 0
ENDPROPERTIES
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
1f
01
ENDCHAR
STARTCHAR 000
ENCODING 64
DWIDTH 8 0
BBX 8 8 0 0
BITMAP
2f
02
ENDCHAR
"#;

        let out = bdf(&chardata.as_bytes());

        assert_eq!(
            out,
            IResult::Done(
                EMPTY,
                BDFFont {
                    metadata: Metadata {
                        version: 2.1,
                        name: String::from("\"open_iconic_all_1x\""),
                        size: (16, 75, 75),
                        bounding_box: (16, 16, 0, 0),
                    },
                    glyphs: vec![
                        Glyph {
                            bitmap: vec![0x1f01],
                            bounding_box: (8, 8, 0, 0),
                            charcode: 64,
                            name: "000".to_string(),
                        },
                        Glyph {
                            bitmap: vec![0x2f02],
                            bounding_box: (8, 8, 0, 0),
                            charcode: 64,
                            name: "000".to_string(),
                        },
                    ],
                }
            )
        );
    }

    //     #[test]
    //     fn it_parses_comments() {
    //         let comment_text = "COMMENT test text\n";
    //         let out = comment(comment_text.as_bytes());

    //         assert_eq!(out, IResult::Done(EMPTY, &b"test text"[..]));

    //         // EMPTY comments
    //         assert_eq!(comment("COMMENT\n".as_bytes()), IResult::Done(EMPTY, EMPTY));
    //     }

    //     #[test]
    //     fn it_parses_bitmap_data() {
    //         assert_eq!(
    //             char_bitmap(&b"BITMAP\n7e\nENDCHAR"[..]),
    //             IResult::Done(EMPTY, vec![0x7e])
    //         );
    //         assert_eq!(
    //             char_bitmap(&b"BITMAP\nff\nENDCHAR"[..]),
    //             IResult::Done(EMPTY, vec![255])
    //         );
    //         assert_eq!(
    //             char_bitmap(&b"BITMAP\nCCCC\nENDCHAR"[..]),
    //             IResult::Done(EMPTY, vec![0xcccc])
    //         );
    //         assert_eq!(
    //             char_bitmap(&b"BITMAP\nffffffff\nENDCHAR"[..]),
    //             IResult::Done(EMPTY, vec![0xffffffff])
    //         );
    //         assert_eq!(
    //             char_bitmap(&b"BITMAP\nffffffff\naaaaaaaa\nENDCHAR"[..]),
    //             IResult::Done(EMPTY, vec![0xffffffff, 0xaaaaaaaa])
    //         );
    //         assert_eq!(
    //             char_bitmap(&b"BITMAP\nff\nff\nff\nff\naa\naa\naa\naa\nENDCHAR"[..]),
    //             IResult::Done(EMPTY, vec![0xffffffff, 0xaaaaaaaa])
    //         );
    //         assert_eq!(
    //             char_bitmap(
    //                 &b"BITMAP\n00\n00\n00\n00\n18\n24\n24\n42\n42\n7E\n42\n42\n42\n42\n00\n00\nENDCHAR"
    //                     [..]
    //             ),
    //             IResult::Done(EMPTY, vec![0x00000000, 0x18242442, 0x427e4242, 0x42420000])
    //         );
    //     }

    //     #[test]
    //     fn it_parses_a_single_char() {
    //         let chardata = r#"STARTCHAR ZZZZ
    // ENCODING 65
    // SWIDTH 500 0
    // DWIDTH 8 0
    // BBX 8 16 0 -2
    // BITMAP
    // 00
    // 00
    // 00
    // 00
    // 18
    // 24
    // 24
    // 42
    // 42
    // 7E
    // 42
    // 42
    // 42
    // 42
    // 00
    // 00
    // ENDCHAR"#;

    //         let out = parse_char(&chardata);

    //         assert_eq!(
    //             out,
    //             IResult::Done(
    //                 EMPTY,
    //                 Glyph {
    //                     name: "ZZZZ".to_string(),
    //                     charcode: 65,
    //                     bitmap: vec![0x00000000, 0x18242442, 0x427e4242, 0x42420000],
    //                     bounding_box: (8, 16, 0, -2),
    //                 }
    //             )
    //         );
    //     }

    //     #[test]
    //     fn it_parses_negative_encodings() {
    //         let chardata = r#"STARTCHAR 000
    // ENCODING -1
    // SWIDTH 432 0
    // DWIDTH 6 0
    // BBX 0 0 0 0
    // BITMAP
    // ENDCHAR"#;

    //         let out = parse_char(&chardata);

    //         assert_eq!(
    //             out,
    //             IResult::Done(
    //                 EMPTY,
    //                 Glyph {
    //                     bitmap: vec![],
    //                     bounding_box: (0, 0, 0, 0),
    //                     charcode: -1i32,
    //                     name: "000".to_string(),
    //                 }
    //             )
    //         );
    //     }

    //     #[test]
    //     fn it_parses_chars_with_no_bitmap() {
    //         let chardata = r#"STARTCHAR 000
    // ENCODING 0
    // SWIDTH 432 0
    // DWIDTH 6 0
    // BBX 0 0 0 0
    // BITMAP
    // ENDCHAR"#;

    //         let out = parse_char(&chardata);

    //         assert_eq!(
    //             out,
    //             IResult::Done(
    //                 EMPTY,
    //                 Glyph {
    //                     bitmap: vec![],
    //                     bounding_box: (0, 0, 0, 0),
    //                     charcode: 0,
    //                     name: "000".to_string(),
    //                 }
    //             )
    //         );
    //     }
}
