#[macro_use]
extern crate nom;

use nom::*;

pub type FontSize = (i32, u32, u32);
pub type BoundingBox = (u32, u32, i32, i32);
type Vec2 = (u32, u32);

#[derive(Debug, Clone)]
pub struct Glyph {
    name: String,
    charcode: i32,
    bounding_box: BoundingBox,
    bitmap: Vec<u8>,
}

#[derive(Debug, Clone)]
struct Properties;

#[derive(Debug, Clone)]
struct Metadata {
    version: f32,
    name: String,
    size: FontSize,
    bounding_box: BoundingBox,
}

#[derive(Debug, Clone)]
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

    pub fn parse(&self) -> Result<(&[u8], BDFFont), nom::Err<&[u8]>> {
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

named!(parse_to_u32<u32>, flat_map!(digit, parse_to!(u32)));

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
    map!(take_until_and_consume!("\n"), |res| {
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
    bitmapdata<Vec<u32>>,
    map!(many_till!(line_of_u32, tag!("ENDCHAR\n")), |res| {
        res.0.into_iter().flat_map(|l| l).collect::<Vec<u32>>()
    })
);

named!(
    glyph<Glyph>,
    do_parse!(
        name: startchar >> charcode: charcode >> swidth >> dwidth >> bounding_box: charboundingbox
            >> tag!("BITMAP\n") >> bitmap: bitmapdata >> ({
            Glyph {
                name,
                charcode,
                bounding_box,
                bitmap: vec![],
            }
        })
    )
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
        endfont
    )
);

pub fn parse_char(input: &str) -> Result<(&[u8], Glyph), nom::Err<&[u8]>> {
    glyph(input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_comments() {
        let empty: &[u8] = &[];

        let comment_text = "COMMENT test text\n";
        let out = comment(comment_text.as_bytes());

        // match out {
        //     Ok((rest, result)) => {
        //         println!("Rest: {:?}", String::from_utf8(rest.to_vec()).unwrap());
        //         println!("Result: {:?}", String::from_utf8(result.to_vec()).unwrap());
        //         assert_eq!(rest.len(), 0);
        //     }
        //     Err(err) => match err {
        //         nom::Err::Incomplete(need) => panic!("Incomplete, need {:?} more", need),
        //         nom::Err::Error(Context::Code(c, error_kind)) => {
        //             println!("Debug: {:?}", String::from_utf8(c.to_vec()).unwrap());

        //             panic!("Parse error {:?}", error_kind);
        //         }
        //         nom::Err::Failure(_) => panic!("Unrecoverable parse error"),
        //         nom::Err::Error(l) => panic!("Idk {:?}", l),
        //     },
        // };

        assert_eq!(out, Ok((empty, &b"test text"[..])));

        // Empty comments
        assert_eq!(comment("COMMENT\n".as_bytes()), Ok((empty, empty)));
    }

    #[test]
    fn it_parses_a_line_of_bitmap_data() {
        let empty: &[u8] = &[];

        assert_eq!(line_of_u32(&b"7e\n"[..]), Ok((empty, vec![0x7e])));
        assert_eq!(line_of_u32(&b"ff\n"[..]), Ok((empty, vec![255])));
        assert_eq!(line_of_u32(&b"CCCC\n"[..]), Ok((empty, vec![0xcccc])));
        assert_eq!(
            line_of_u32(&b"ffffffff\n"[..]),
            Ok((empty, vec![0xffffffff]))
        );
        assert_eq!(
            line_of_u32(&b"ffffffffaaaaaaaa\n"[..]),
            Ok((empty, vec![0xffffffff, 0xaaaaaaaa]))
        );
    }

    #[test]
    fn it_parses_a_single_char() {
        let chardata = r#"STARTCHAR U+0041
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
"#;

        let out = parse_char(&chardata);

        match out {
            Ok((rest, _)) => {
                println!("Rest: {:?}", String::from_utf8(rest.to_vec()).unwrap());
                assert_eq!(rest.len(), 0);
            }
            Err(err) => match err {
                nom::Err::Incomplete(need) => panic!("Incomplete, need {:?} more", need),
                nom::Err::Error(Context::Code(c, error_kind)) => {
                    println!("Debug: {:?}", String::from_utf8(c.to_vec()).unwrap());

                    panic!("Parse error {:?}", error_kind);
                }
                nom::Err::Failure(_) => panic!("Unrecoverable parse error"),
                nom::Err::Error(l) => panic!("Idk {:?}", l),
            },
        };
    }

    #[test]
    fn it_parses_negative_encodings() {
        let chardata = r#"STARTCHAR U+0041
ENCODING -1
SWIDTH 500 0
DWIDTH 8 0
BBX 8 16 0 -2
BITMAP
ff
ENDCHAR
"#;

        let out = parse_char(&chardata);

        match out {
            Err(err) => match err {
                nom::Err::Incomplete(need) => panic!("Incomplete, need {:?} more", need),
                nom::Err::Error(Context::Code(c, error_kind)) => {
                    println!("Debug: {:?}", String::from_utf8(c.to_vec()).unwrap());

                    panic!("Parse error {:?}", error_kind);
                }
                nom::Err::Failure(_) => panic!("Unrecoverable parse error"),
                nom::Err::Error(l) => panic!("Idk {:?}", l),
            },
            Ok((rest, glyph)) => {
                assert_eq!(glyph.charcode, -1i32);

                assert_eq!(rest.len(), 0);
            }
        }
    }

    #[test]
    fn it_parses_chars_with_no_bitmap() {
        let chardata = r#"STARTCHAR 000
ENCODING 0
SWIDTH 432 0
DWIDTH 6 0
BBX 0 0 0 0
BITMAP
ENDCHAR
"#;

        let out = parse_char(&chardata);

        match out {
            Err(err) => match err {
                nom::Err::Incomplete(need) => panic!("Incomplete, need {:?} more", need),
                nom::Err::Error(Context::Code(c, error_kind)) => {
                    println!("Debug: {:?}", String::from_utf8(c.to_vec()).unwrap());

                    panic!("Parse error {:?}", error_kind);
                }
                nom::Err::Failure(_) => panic!("Unrecoverable parse error"),
                nom::Err::Error(l) => panic!("Idk {:?}", l),
            },
            Ok((rest, glyph)) => {
                assert_eq!(glyph.bitmap.len(), 0);
                assert_eq!(rest.len(), 0);
            }
        }
    }
}
