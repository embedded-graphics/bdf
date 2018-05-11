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

named!(startfont<f32>, ws!(preceded!(tag!("STARTFONT"), float)));

named!(
    fontname<String>,
    flat_map!(
        ws!(preceded!(tag!("FONT"), take_until!("\n"))),
        parse_to!(String)
    )
);

named!(
    size<FontSize>,
    ws!(preceded!(
        tag!("SIZE"),
        tuple!(parse_to_i32, parse_to_u32, parse_to_u32)
    ))
);

named!(
    boundingbox<BoundingBox>,
    ws!(tuple!(
        parse_to_u32,
        parse_to_u32,
        parse_to_i32,
        parse_to_i32
    ))
);

named!(
    fontboundingbox<BoundingBox>,
    ws!(preceded!(tag!("FONTBOUNDINGBOX"), boundingbox))
);

named!(
    comment,
    delimited!(
        alt!(tag!("COMMENT ") | tag!("COMMENT")),
        take_until!("\n"),
        line_ending
    )
);

named!(
    startproperties<u32>,
    ws!(preceded!(tag!("STARTPROPERTIES"), parse_to_u32))
);

named!(
    fontascent<u32>,
    ws!(preceded!(tag!("FONT_ASCENT"), parse_to_u32))
);

named!(
    fontdescent<u32>,
    ws!(preceded!(tag!("FONT_DESCENT"), parse_to_u32))
);

named!(numchars<u32>, ws!(preceded!(tag!("CHARS"), parse_to_u32)));

named!(
    startchar<String>,
    flat_map!(
        ws!(preceded!(tag!("STARTCHAR"), take_until!("\n"))),
        parse_to!(String)
    )
);

named!(
    charcode<i32>,
    ws!(preceded!(tag!("ENCODING"), parse_to_i32))
);

named!(
    swidth<Vec2>,
    ws!(preceded!(
        tag!("SWIDTH"),
        tuple!(parse_to_u32, parse_to_u32)
    ))
);

named!(
    dwidth<Vec2>,
    ws!(preceded!(
        tag!("DWIDTH"),
        tuple!(parse_to_u32, parse_to_u32)
    ))
);

named!(
    charboundingbox<BoundingBox>,
    ws!(preceded!(tag!("BBX"), boundingbox))
);

named!(bitmap, terminated!(tag!("BITMAP"), line_ending));
named!(
    endproperties,
    terminated!(tag!("ENDPROPERTIES"), line_ending)
);
named!(endchar, terminated!(tag!("ENDCHAR"), line_ending));
named!(endfont, terminated!(tag!("ENDFONT"), line_ending));

named!(
    chardata<Vec<u8>>,
    many0!(flat_map!(take!(2), parse_to!(u8)))
);

named!(
    metadata<Metadata>,
    ws!(do_parse!(
        many0!(comment) >> version: startfont >> many0!(comment) >> name: fontname
            >> many0!(comment) >> size: size >> many0!(comment)
            >> bounding_box: fontboundingbox >> ({
            Metadata {
                version,
                name,
                size,
                bounding_box,
            }
        })
    ))
);

// TODO: Properties
named!(
    properties<Properties>,
    map!(
        ws!(preceded!(
            startproperties,
            take_until_and_consume!("ENDPROPERTIES")
        )),
        |_| Properties
    )
);

named!(
    line_of_u32<Vec<u32>>,
    map!(ws!(take_until_and_consume!("ENDCHAR")), |res| {
        println!("{:?}", res);
        res.chunks(8)
            .map(|c| {
                c.iter()
                    .rev()
                    .enumerate()
                    .map(|(k, &v)| {
                        let digit = v as char;
                        digit.to_digit(16).unwrap_or(0) << (k * 4)
                    })
                    .sum()
            })
            .collect()
    })
);

named!(
    char_bitmap<Vec<u32>>,
    map!(
        ws!(delimited!(
            tag!("BITMAP"),
            many0!(take_until!("\n")),
            tag!("ENDCHAR")
        )),
        |res| {
            res.iter()
                .flat_map(|&l| l)
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
    ws!(do_parse!(
        name: startchar >> charcode: charcode >> opt!(swidth) >> opt!(dwidth)
            >> bounding_box: charboundingbox >> bitmap: char_bitmap >> ({
            Glyph {
                name,
                charcode,
                bounding_box,
                bitmap,
            }
        })
    ))
);

named!(
    bdf<BDFFont>,
    terminated!(
        do_parse!(
            metadata: metadata >> many0!(comment) >> properties >> opt!(numchars)
                >> glyphs: many1!(ws!(glyph)) >> ({
                BDFFont {
                    metadata,
                    glyphs: glyphs,
                }
            })
        ),
        opt!(endfont)
    )
);

pub fn parse_char(input: &str) -> nom::IResult<&[u8], Glyph> {
    glyph(input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult;

    const EMPTY: &[u8] = &[];

    //     #[test]
    //     fn it_parses_optional_endfont_tag() {
    //         let chardata = r#"STARTFONT 2.1
    // FONT "open_iconic_all_1x"
    // SIZE 16 75 75
    // FONTBOUNDINGBOX 16 16 0 0
    // STARTPROPERTIES 3
    // COPYRIGHT "https://github.com/iconic/open-iconic, SIL OPEN FONT LICENSE"
    // FONT_ASCENT 0
    // FONT_DESCENT 0
    // ENDPROPERTIES
    // STARTCHAR /home/kraus/git/open-iconic/png/account-login.png
    // ENCODING 64
    // DWIDTH 8 0
    // BBX 8 8 0 0
    // BITMAP
    // 1f
    // 01
    // 09
    // fd
    // 09
    // 01
    // 1f
    // 00
    // ENDCHAR
    // ENDFONT
    // "#;

    //         let out = bdf(&chardata.as_bytes());

    //         assert_eq!(out, IResult::Incomplete(Needed::Size(10)));
    //     }

    #[test]
    fn it_parses_comments() {
        let comment_text = "COMMENT test text\n";
        let out = comment(comment_text.as_bytes());

        assert_eq!(out, IResult::Done(EMPTY, &b"test text"[..]));

        // EMPTY comments
        assert_eq!(comment("COMMENT\n".as_bytes()), IResult::Done(EMPTY, EMPTY));
    }

    #[test]
    fn it_parses_bitmap_data() {
        assert_eq!(
            char_bitmap(&b"BITMAP\n7e\nENDCHAR"[..]),
            IResult::Done(EMPTY, vec![0x7e])
        );
        assert_eq!(
            char_bitmap(&b"BITMAP\nff\nENDCHAR"[..]),
            IResult::Done(EMPTY, vec![255])
        );
        assert_eq!(
            char_bitmap(&b"BITMAP\nCCCC\nENDCHAR"[..]),
            IResult::Done(EMPTY, vec![0xcccc])
        );
        assert_eq!(
            char_bitmap(&b"BITMAP\nffffffff\nENDCHAR"[..]),
            IResult::Done(EMPTY, vec![0xffffffff])
        );
        assert_eq!(
            char_bitmap(&b"BITMAP\nffffffff\naaaaaaaa\nENDCHAR"[..]),
            IResult::Done(EMPTY, vec![0xffffffff, 0xaaaaaaaa])
        );
        assert_eq!(
            char_bitmap(&b"BITMAP\nff\nff\nff\nff\naa\naa\naa\naa\nENDCHAR"[..]),
            IResult::Done(EMPTY, vec![0xffffffff, 0xaaaaaaaa])
        );
        assert_eq!(
            char_bitmap(
                &b"BITMAP\n00\n00\n00\n00\n18\n24\n24\n42\n42\n7E\n42\n42\n42\n42\n00\n00\nENDCHAR"
                    [..]
            ),
            IResult::Done(EMPTY, vec![0x00000000, 0x18242442, 0x427e4242, 0x42420000])
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

        let out = parse_char(&chardata);

        assert_eq!(
            out,
            IResult::Done(
                EMPTY,
                Glyph {
                    name: "ZZZZ".to_string(),
                    charcode: 65,
                    bitmap: vec![0x00000000, 0x18242442, 0x427e4242, 0x42420000],
                    bounding_box: (8, 16, 0, -2),
                }
            )
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

        let out = parse_char(&chardata);

        assert_eq!(
            out,
            IResult::Done(
                EMPTY,
                Glyph {
                    bitmap: vec![],
                    bounding_box: (0, 0, 0, 0),
                    charcode: -1i32,
                    name: "000".to_string(),
                }
            )
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

        let out = parse_char(&chardata);

        assert_eq!(
            out,
            IResult::Done(
                EMPTY,
                Glyph {
                    bitmap: vec![],
                    bounding_box: (0, 0, 0, 0),
                    charcode: 0,
                    name: "000".to_string(),
                }
            )
        );
    }
}
